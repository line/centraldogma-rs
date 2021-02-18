#[macro_use]
mod utils;
use centraldogma as cd;

use anyhow::{ensure, Context, Result};
use futures::future::{Future, FutureExt};
use std::pin::Pin;

struct TestContext {
    client: cd::Client,
    project: cd::Project,
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
    let client = cd::Client::new_with_token("http://localhost:36462", None)
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

    Ok(TestContext { client, project })
}

async fn teardown(ctx: TestContext) -> Result<()> {
    cd::project::remove(&ctx.client, &ctx.project.name)
        .await
        .context("Failed to remove the project")?;

    cd::project::purge(&ctx.client, &ctx.project.name)
        .await
        .context("Failed to purge the project")?;

    Ok(())
}

fn t1<'a>(ctx: &'a mut TestContext) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
    async move {
        // List repositories
        let repos = cd::repository::list_by_project_name(&ctx.client, &ctx.project.name)
            .await
            .context("Failed to list repositories from project")?;
            ensure!(repos.len() == 2, here!("New project should have 2 repos"));

        // Create new repository
        let repo_name = "TestRepo";
        let new_repo = cd::repository::create(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to create new Repository")?;
        ensure!(repo_name == new_repo.name, here!("Wrong repo name"));

        // Remove created repository
        cd::repository::remove(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to remove Repository")?;

        let removed_repos =
            cd::repository::list_removed_by_project_name(&ctx.client, &ctx.project.name)
                .await
                .context("Failed to list removed repositories")?;

        let mut found = false;
        for repo in removed_repos.iter() {
            if repo.name == repo_name {
                found = true;
            }
        }
        ensure!(found, here!("Removed repo not showed in removed repo list"));

        // Unremove removed repository
        let unremoved_repo = cd::repository::unremove(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to unremove removed Repository")?;
        ensure!(unremoved_repo.name == repo_name, here!("Invalid unremove"));

        let repos = cd::repository::list_by_project_name(&ctx.client, &ctx.project.name)
            .await
            .context("Failed to list repositories from project")?;

        let mut found = false;
        for repo in repos.iter() {
            if repo.name == repo_name {
                found = true;
            }
        }
        ensure!(found, here!("Unremoved repo not showed in repo list"));

        cd::repository::remove(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to remove Repository")?;

        // Purge removed repository
        cd::repository::purge(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to purge removed Repository")?;

        let removed_repos =
            cd::repository::list_removed_by_project_name(&ctx.client, &ctx.project.name)
                .await
                .context("Failed to list removed repositories")?;

        let mut found = false;
        for repo in removed_repos.iter() {
            if repo.name == repo_name {
                found = true;
            }
        }
        ensure!(!found, here!("Purged repo showed in removed repo list"));

        Ok(())
    }
    .boxed()
}

#[cfg(test)]
#[tokio::test]
async fn test_repos() {
    run_test(t1).await
}
