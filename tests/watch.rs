#[macro_use]
mod utils;

use cd::{
    model::{Change, ChangeContent, CommitMessage, EntryContent, Query, Revision},
    ContentService, ProjectService, RepoService, WatchService,
};
use centraldogma as cd;

use std::{pin::Pin, time::Duration};

use anyhow::{ensure, Context, Result};
use futures::{
    future::{Future, FutureExt},
    StreamExt,
};
use serde_json::json;

struct TestContext {
    client: cd::Client,
    project: cd::model::Project,
    repo: cd::model::Repository,
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

fn watch_file_stream_test<'a>(
    ctx: &'a mut TestContext,
) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
    async move {
        let r = ctx.client.repo(&ctx.project.name, &ctx.repo.name);

        let commit_msg = CommitMessage {
            summary: "File".to_string(),
            detail: None,
        };
        let file_change = vec![Change {
            path: "/a.json".to_string(),
            content: ChangeContent::UpsertJson(json!({"a": "b"})),
        }];

        r.push(Revision::HEAD, commit_msg, file_change)
            .await
            .context(here!("Failed to push file"))?;

        let watch_stream = r
            .watch_file_stream(&Query::of_json("/a.json").unwrap())
            .context(here!("Failed to get file watch stream"))?;

        let new_commit_msg = CommitMessage {
            summary: "change content".to_string(),
            detail: None,
        };
        let new_change = vec![Change {
            path: "/a.json".to_string(),
            content: ChangeContent::UpsertJson(json!({"a": "c"})),
        }];
        let new_push = async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            r.push(Revision::HEAD, new_commit_msg, new_change).await
        };

        let sleep = tokio::time::sleep(Duration::from_millis(10000));
        futures::pin_mut!(sleep);

        let mut s = watch_stream.take_until(sleep);
        let (wr, _) = tokio::join!(s.next(), new_push);

        println!("Watch result: {:?}", wr);
        ensure!(wr.is_some(), here!("Failed to get initial watch result"));
        let wr = wr.unwrap();

        ensure!(
            wr.entry.path == "/a.json",
            here!("Wrong entry path returned")
        );
        ensure!(
            matches!(wr.entry.content, EntryContent::Json(json) if json == json!({"a": "c"})),
            here!("Wrong entry content returned")
        );

        Ok(())
    }
    .boxed()
}

fn watch_repo_stream_test<'a>(
    ctx: &'a mut TestContext,
) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
    async move {
        let r = ctx.client.repo(&ctx.project.name, &ctx.repo.name);

        let watch_stream = r
            .watch_repo_stream("")
            .context(here!("Failed to get file watch stream"))?;

        let new_commit_msg = CommitMessage {
            summary: "change content".to_string(),
            detail: None,
        };
        let new_change = vec![Change {
            path: "/a.json".to_string(),
            content: ChangeContent::UpsertJson(json!({"a": "c"})),
        }];
        let new_push = async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            r.push(Revision::HEAD, new_commit_msg, new_change).await
        };

        let sleep = tokio::time::sleep(Duration::from_millis(10000));
        futures::pin_mut!(sleep);

        let mut s = watch_stream.take_until(sleep);
        let (wr, _) = tokio::join!(s.next(), new_push);

        println!("Watch result: {:?}", wr);
        ensure!(wr.is_some(), here!("Failed to get initial watch result"));

        Ok(())
    }
    .boxed()
}

#[cfg(test)]
#[tokio::test]
async fn test_watch() {
    run_test(watch_file_stream_test).await;
    run_test(watch_repo_stream_test).await;
}
