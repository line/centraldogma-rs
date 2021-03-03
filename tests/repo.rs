#[macro_use]
mod utils;
use cd::{ProjectService, RepoService};
use centraldogma as cd;

use anyhow::{ensure, Context, Result};
use futures::future::{Future, FutureExt};
use std::pin::Pin;

struct TestContext {
    client: cd::Client,
    project: cd::model::Project,
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

    Ok(TestContext { client, project })
}

async fn teardown(ctx: TestContext) -> Result<()> {
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

fn t1<'a>(ctx: &'a mut TestContext) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
    async move {
        let r = ctx.client.project(&ctx.project.name);

        // List repositories
        let repos = r
            .list_repos()
            .await
            .context("Failed to list repositories from project")?;
        ensure!(repos.len() == 2, here!("New project should have 2 repos"));

        // Create new repository
        let repo_name = "TestRepo";
        let new_repo = r
            .create_repo(repo_name)
            .await
            .context("Failed to create new Repository")?;
        ensure!(repo_name == new_repo.name, here!("Wrong repo name"));

        // Remove created repository
        r.remove_repo(repo_name)
            .await
            .context("Failed to remove Repository")?;

        let removed_repos = r
            .list_removed_repos()
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
        let unremoved_repo = r
            .unremove_repo(repo_name)
            .await
            .context("Failed to unremove removed Repository")?;
        ensure!(unremoved_repo.name == repo_name, here!("Invalid unremove"));

        let repos = r
            .list_repos()
            .await
            .context("Failed to list repositories from project")?;

        let mut found = false;
        for repo in repos.iter() {
            if repo.name == repo_name {
                found = true;
            }
        }
        ensure!(found, here!("Unremoved repo not showed in repo list"));

        r.remove_repo(repo_name)
            .await
            .context("Failed to remove Repository")?;

        // Purge removed repository
        r.purge_repo(repo_name)
            .await
            .context("Failed to purge removed Repository")?;

        let removed_repos = r
            .list_removed_repos()
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
