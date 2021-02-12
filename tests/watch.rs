#[macro_use]
mod utils;

use centraldogma as cd;
use cd::{
    Change, ChangeContent, CommitDetail, CommitMessage, Entry, EntryContent, Query,
    QueryType,
};

use std::{panic, pin::Pin, time::Duration};

use anyhow::{bail, Context, Result};
use futures::{StreamExt, future::{Future, FutureExt}};
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

fn watch_test<'a>(ctx: &'a mut TestContext) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
    async move {
        let commit_msg = CommitMessage {
            summary: "File".to_string(),
            detail: None
        };
        let file_change = vec![Change {
            path: "/a.json".to_string(),
            content: ChangeContent::UpsertJson(json!({"a": "b"})),
        }];

        cd::content::push(
            &ctx.client,
            &ctx.project.name,
            &ctx.repo.name,
            -1,
            commit_msg,
            file_change,
        )
        .await
        .context(here!("Failed to push file"))?;

        let query = Query {
            path: "/a.json".to_string(),
            r#type: QueryType::Identity,
        };
        let watch_stream = cd::watch::watch_file_stream(
            ctx.client.clone(),
            &ctx.project.name,
            &ctx.repo.name,
            &query
        )
        .context(here!("Failed to get file watch stream"))?;
        futures::pin_mut!(watch_stream);

        let new_commit_msg = CommitMessage {
            summary: "change content".to_string(),
            detail: None,
        };
        let new_change = vec![Change {
            path: "/a.json".to_string(),
            content: ChangeContent::UpsertJson(json!({"a": "c"})),
        }];
        let new_push = async move {
            tokio::time::sleep(Duration::from_millis(1)).await;
            cd::content::push(
                &ctx.client,
                &ctx.project.name,
                &ctx.repo.name,
                -1,
                new_commit_msg,
                new_change
            ).await
        };

        let sleep = tokio::time::sleep(Duration::from_millis(10000));
        futures::pin_mut!(sleep);

        let mut s = watch_stream.take_until(sleep);
        let (wr, _) = tokio::join!(s.next(), new_push);

        println!("Watch result: {:?}", wr);
        match wr {
            Some(wr) => {
                match wr.entry {
                    Some(entry) => {
                        if entry.path != "/a.json" {
                            bail!(here!("Wrong entry path returned"));
                        }
                        if let EntryContent::Json(json) = entry.content {
                            if json != json!({"a": "c"}) {
                                bail!(here!("Wrong entry content returned"));
                            }
                        } else {
                            bail!(here!("Wrong entry content returned"));
                        }
                    }
                    None => bail!(here!("Empty entry")),
                }

            },
            None => bail!(here!("Failed to get initial watch result")),
        }

        Ok(())
    }
    .boxed()
}

#[cfg(test)]
#[tokio::test]
async fn test_watch() {
    run_test(watch_test).await;
}
