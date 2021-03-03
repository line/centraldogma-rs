#[macro_use]
mod utils;

use cd::{
    model::{
        Change, ChangeContent, CommitDetail, CommitMessage, Entry, EntryContent, Project, Query,
        Repository, Revision,
    },
    ContentService, ProjectService, RepoService,
};
use centraldogma as cd;

use std::pin::Pin;

use anyhow::{ensure, Context, Result};
use futures::future::{Future, FutureExt};
use serde_json::json;

struct TestContext {
    client: cd::Client,
    project: Project,
    repo: Repository,
}

async fn run_test<T>(test: T)
where
    for<'a> T: FnOnce(&'a mut TestContext) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>,
{
    let mut ctx = setup().await.expect("Failed to setup for test");

    let result = test(&mut ctx).await;

    teardown(ctx).await.expect("Failed to teardown test setup");

    result.unwrap();
}

async fn setup() -> Result<TestContext> {
    let client = cd::Client::new("http://localhost:36462", None)
        .await
        .context("Failed to create client")?;
    let projects = client
        .list_projects()
        .await
        .context("Failed to list projects")?;
    assert_eq!(0, projects.len());

    let prj_name = "TestProject";
    let project = client
        .create_project(prj_name)
        .await
        .context("Failed to create new project")?;

    let repo_name = "TestRepo";
    let repo = client
        .project(prj_name)
        .create_repo(repo_name)
        .await
        .context("Failed to create new repository")?;

    Ok(TestContext {
        client,
        project,
        repo,
    })
}

async fn teardown(ctx: TestContext) -> Result<()> {
    ctx.client
        .project(&ctx.project.name)
        .remove_repo(&ctx.repo.name)
        .await
        .context("Failed to remove the repo")?;

    ctx.client
        .project(&ctx.project.name)
        .purge_repo(&ctx.repo.name)
        .await
        .context("Failed to remove the repo")?;

    ctx.client
        .remove_project(&ctx.project.name)
        .await
        .context("Failed to remove the project")?;

    ctx.client
        .purge_project(&ctx.project.name)
        .await
        .context("Failed to purge the project")?;

    Ok(())
}

fn t<'a>(ctx: &'a mut TestContext) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
    async move {
        let r = ctx.client.repo(&ctx.project.name, &ctx.repo.name);

        // Push data
        let push_result = {
            let commit_msg = CommitMessage {
                summary: "New file".to_string(),
                detail: Some(CommitDetail::Plaintext("detail".to_string())),
            };
            let changes = vec![Change {
                path: "/a.json".to_string(),
                content: ChangeContent::UpsertJson(json!({
                    "test_key": "test_value"
                })),
            }, Change {
                path: "/folder/b.txt".to_string(),
                content: ChangeContent::UpsertText("text value".to_string()),
            }];

            r.push(
                Revision::HEAD,
                commit_msg,
                changes,
            )
            .await
            .context(here!("Failed to push file"))?
        };

        // Get single file
        {
            let file: Entry = r.get_file(
                push_result.revision,
                &Query::of_json("/a.json").unwrap(),
            )
            .await
            .context(here!("Failed to fetch file content"))?;

            println!("File: {:?}", &file);
            ensure!(
                matches!(file.content, EntryContent::Json(json) if json == json!({"test_key": "test_value"})),
                here!("Expect same json content")
            );
        }

        // List files
        {
            let file_list = r.list_files(
                Revision::HEAD,
                ""
            )
            .await
            .context(here!("Failed to list files"))?;

            println!("File list: {:?}", &file_list);

            ensure!(
                file_list.len() == 3,
                here!("Wrong number of file entry returned")
            );
        }

        // Get single file jsonpath
        {
            let file: Entry = r.get_file(
                push_result.revision,
                &Query::of_json_path("/a.json", vec!["test_key".to_owned()]).unwrap(),
            )
            .await
            .context(here!("Failed to fetch file content"))?;
            println!("File: {:?}", &file);

            ensure!(
                matches!(file.content, EntryContent::Json(json) if json == json!("test_value")),
                here!("Expect same json content")
            );
        }

        // Get history
        {
            let commits = r.get_history(
                Revision::INIT,
                Revision::HEAD,
                "/**",
                20
            )
            .await
            .context(here!("Failed to get history"))?;

            println!("History: {:?}", &commits);
        }

        // Get multiple files
        {
            let entries = r.get_files(
                push_result.revision,
                "a*"
            )
            .await
            .context(here!("Failed to fetch multiple files"))?;
            ensure!(entries.len() == 1, here!("wrong number of entries returned"));

            let entries = r.get_files(
                push_result.revision,
                "*"
            )
            .await
            .context(here!("Failed to fetch multiple files"))?;
            ensure!(entries.len() == 3, here!("wrong number of entries returned"));

            println!("Entries: {:?}", &entries);
            let exist = entries.iter().any(|e| {
                e.path == "/folder/b.txt" && matches!(&e.content, EntryContent::Text(s) if s == "text value\n")
            });
            ensure!(exist, here!("Expected value not found"));
        }

        // Get file diff
        {
            let commit_msg = CommitMessage {
                summary: "Update a.json".to_string(),
                detail: None,
            };
            let changes = vec![Change {
                path: "/a.json".to_string(),
                content: ChangeContent::ApplyJsonPatch(json!([
                    {"op": "replace", "path": "/test_key", "value": "updated_value"},
                    {"op": "add", "path": "/new_key", "value": ["new_array_item1", "new_array_item2"]}
                ])),
            }];

            r.push(
                Revision::HEAD,
                commit_msg,
                changes,
            )
            .await
            .context(here!("Failed to push file"))?;

            let diff = r.get_diff(
                Revision::from(1),
                Revision::HEAD,
                &Query::of_json("/a.json").unwrap(),
            )
            .await
            .context(here!("Failed to get diff"))?;
            println!("Diff: {:?}", diff);

            ensure!(diff.path == "/a.json", here!("Diff path incorrect"));

            let expected_json = json!({
                "new_key": ["new_array_item1", "new_array_item2"],
                "test_key": "updated_value"
            });
            ensure!(
                matches!(diff.content, ChangeContent::UpsertJson(json) if json == expected_json),
                here!("Diff content incorrect")
            );
        }


        // Get multiple file diff
        {
            let diffs = r.get_diffs(
                Revision::from(1),
                Revision::HEAD,
                "*"
            )
            .await
            .context(here!("Failed to get diff"))?;

            ensure!(diffs.len() == 2, here!("Expect 2 diffs"));
        }

        Ok(())
    }
    .boxed()
}

#[cfg(test)]
#[tokio::test]
async fn test_content() {
    run_test(t).await;
}
