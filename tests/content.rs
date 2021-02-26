#[macro_use]
mod utils;

use cd::{
    Change, ChangeContent, CommitDetail, CommitMessage, ContentService, Entry, EntryContent, Query, Revision,
};
use centraldogma as cd;

use std::pin::Pin;

use anyhow::{ensure, Context, Result};
use futures::future::{Future, FutureExt};
use serde_json::json;

struct TestContext {
    client: cd::Client,
    project: cd::Project,
    repo: cd::Repository,
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
    let client = cd::Client::from_token("http://localhost:36462", None)
        .await
        .context("Failed to create client")?;
    let projects = cd::project::list(&client)
        .await
        .context("Failed to list projects")?;
    assert_eq!(0, projects.len());

    let prj_name = "TestProject";
    let project = cd::project::create(&client, prj_name)
        .await
        .context("Failed to create new project")?;

    let repo_name = "TestRepo";
    let repo = cd::repository::create(&client, prj_name, repo_name)
        .await
        .context("Failed to create new repository")?;

    Ok(TestContext {
        client,
        project,
        repo,
    })
}

async fn teardown(ctx: TestContext) -> Result<()> {
    cd::repository::remove(&ctx.client, &ctx.project.name, &ctx.repo.name)
        .await
        .context("Failed to remove the repo")?;

    cd::repository::purge(&ctx.client, &ctx.project.name, &ctx.repo.name)
        .await
        .context("Failed to remove the repo")?;

    cd::project::remove(&ctx.client, &ctx.project.name)
        .await
        .context("Failed to remove the project")?;

    cd::project::purge(&ctx.client, &ctx.project.name)
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
                path: "/b.txt".to_string(),
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
                &Query::of_json("/a.json"),
            )
            .await
            .context(here!("Failed to fetch file content"))?;

            println!("File: {:?}", &file);
            ensure!(
                matches!(file.content, EntryContent::Json(json) if json == json!({"test_key": "test_value"})),
                here!("Expect same json content")
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
            ensure!(entries.len() == 2, here!("wrong number of entries returned"));

            println!("Entries: {:?}", &entries);
            let exist = entries.iter().any(|e| {
                e.path == "/b.txt" && matches!(&e.content, EntryContent::Text(s) if s == "text value\n")
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
                &Query::identity("/a.json"),
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
