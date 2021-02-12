#[macro_use]
mod utils;

use centraldogma as cd;
use cd::{
    Change, ChangeContent, CommitDetail, CommitMessage, Entry, EntryContent, Query,
    QueryType,
};

use std::{panic, pin::Pin};

use anyhow::{bail, Context, Result};
use futures::future::{Future, FutureExt};
use serde_json::json;

struct TestContext {
    client: cd::Client,
    project: cd::Project,
    repo: cd::Repository,
}

async fn run_test<T>(test: T) -> ()
where
    for<'a> T: FnOnce(&'a mut TestContext) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>,
{
    let mut ctx = setup().await.expect("Failed to setup for test");

    let result = test(&mut ctx).await;

    teardown(ctx).await.expect("Failed to teardown test setup");

    result.unwrap();
}

async fn setup() -> Result<TestContext> {
    let client = cd::Client::new_with_token("http://localhost:36462".to_string(), None)
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

            cd::content::push(
                &ctx.client,
                &ctx.project.name,
                &ctx.repo.name,
                -1,
                commit_msg,
                changes,
            )
            .await
            .context(here!("Failed to push file"))?
        };

        // Get single file
        {
            let file_query = Query {
                path: "/a.json".to_string(),
                r#type: QueryType::Identity,
            };
            let file: Entry = cd::content::get_file(
                &ctx.client,
                &ctx.project.name,
                &ctx.repo.name,
                push_result.revision,
                &file_query,
            )
            .await
            .context(here!("Failed to fetch file content"))?;

            println!("File: {:?}", &file);
            if let EntryContent::Json(json) = file.content {
                if json != json!({"test_key": "test_value"}) {
                    bail!(here!("Expect same json content"));
                }
            } else {
                bail!(here!("Expect json content"));
            }
        }

        // Get single file jsonpath
        {
            let file_query = Query {
                path: "/a.json".to_string(),
                r#type: QueryType::JsonPath(vec!["test_key".to_string()]),
            };
            let file: Entry = cd::content::get_file(
                &ctx.client,
                &ctx.project.name,
                &ctx.repo.name,
                push_result.revision,
                &file_query,
            )
            .await
            .context(here!("Failed to fetch file content"))?;
            println!("File: {:?}", &file);
            if let EntryContent::Json(json) = file.content {
                if json != json!("test_value") {
                    bail!(here!("Expect same json content"));
                }
            } else {
                bail!(here!("Expect json content"));
            }
        }

        // Get multiple files
        {
            let entries = cd::content::get_files(
                &ctx.client,
                &ctx.project.name,
                &ctx.repo.name,
                push_result.revision,
                "a*"
            )
            .await
            .context(here!("Failed to fetch multiple files"))?;
            if entries.len() != 1 {
                bail!(here!("wrong number of entries returned"));
            }

            let entries = cd::content::get_files(
                &ctx.client,
                &ctx.project.name,
                &ctx.repo.name,
                push_result.revision,
                "*"
            )
            .await
            .context(here!("Failed to fetch multiple files"))?;
            if entries.len() != 2 {
                bail!(here!("wrong number of entries returned"));
            }

            println!("Entries: {:?}", &entries);
            let exist = entries.iter().any(|e| {
                e.path == "/b.txt" && matches!(&e.content, EntryContent::Text(s) if s == "text value\n")
            });
            if !exist {
                bail!(here!("Expected value not found"));
            }
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

            cd::content::push(
                &ctx.client,
                &ctx.project.name,
                &ctx.repo.name,
                -1,
                commit_msg,
                changes,
            )
            .await
            .context(here!("Failed to push file"))?;

            let query = Query {
                path: "/a.json".to_string(),
                r#type: QueryType::Identity,
            };
            let diff = cd::content::get_diff(
                &ctx.client,
                &ctx.project.name,
                &ctx.repo.name,
                "1",
                "-1",
                &query
            )
            .await
            .context(here!("Failed to get diff"))?;
            println!("Diff: {:?}", diff);

            if diff.path != "/a.json" {
                bail!(here!("Diff path incorrect"));
            }

            let expected_json = json!({
                "new_key": ["new_array_item1", "new_array_item2"],
                "test_key": "updated_value"
            });
            match diff.content {
                ChangeContent::UpsertJson(json) if json == expected_json => {},
                _ => bail!(here!("Diff content incorrect")),
            }
        }


        // Get multiple file diff
        {
            let diffs = cd::content::get_diffs(
                &ctx.client,
                &ctx.project.name,
                &ctx.repo.name,
                "1",
                "-1",
                "*"
            )
            .await
            .context(here!("Failed to get diff"))?;

            if diffs.len() != 2 {
                bail!(here!("Expect 2 diffs"));
            }
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
