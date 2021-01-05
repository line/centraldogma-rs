use centraldogma as cd;

use std::{panic, pin::Pin};
use futures::future::{Future, FutureExt};

use anyhow::{bail, Context, Result};

struct TestContext {
    client: cd::Client,
    project: cd::Project,
}

async fn run_test<T>(test: T) -> ()
where
    for <'a> T: FnOnce(&'a mut TestContext) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>,
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
    let projects = cd::Project::list(&client)
        .await
        .context("Failed to list projects")?;
    assert_eq!(0, projects.len());

    let prj_name = "TestProject";
    let project = cd::Project::create(&client, prj_name)
        .await
        .context("Failed to create new project")?;

    Ok(TestContext {
        client,
        project,
    })
}

async fn teardown(ctx: TestContext) -> Result<()> {
    cd::Project::remove(&ctx.client, &ctx.project.name)
        .await
        .context("Failed to remove the project")?;

    cd::Project::purge(&ctx.client, &ctx.project.name)
        .await
        .context("Failed to purge the project")?;

    Ok(())
}

fn t1<'a>(ctx: &'a mut TestContext) -> Pin<Box<dyn Future<Output = Result<()>> +'a>> {
    async move {
        // List repositories
        let repos = cd::Repository::list_by_project_name(&ctx.client, &ctx.project.name)
            .await
            .context("Failed to list repositories from project")?;
        if repos.len() != 2 {
            bail!("New project should have 2 repos");
        }

        // Create new repository
        let repo_name = "TestRepo";
        let new_repo = cd::Repository::create(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to create new Repository")?;
        if repo_name != new_repo.name {
            bail!("Wrong repo name")
        }

        // Remove created repository
        cd::Repository::remove(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to remove Repository")?;

        let removed_repos = cd::Repository::list_removed_by_project_name(&ctx.client, &ctx.project.name)
            .await
            .context("Failed to list removed repositories")?;

        let mut found = false;
        for repo in removed_repos.iter() {
            if repo.name == repo_name {
                found = true;
            }
        }
        if !found {
            bail!("Removed repo not showed in removed repo list");
        }

        // Unremove removed repository
        let unremoved_repo = cd::Repository::unremove(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to unremove removed Repository")?;
        if unremoved_repo.name != repo_name {
            bail!("Invalid unremove");
        }

        let repos = cd::Repository::list_by_project_name(&ctx.client, &ctx.project.name)
            .await
            .context("Failed to list repositories from project")?;

        let mut found = false;
        for repo in repos.iter() {
            if repo.name == repo_name {
                found = true;
            }
        }
        if !found {
            bail!("Unremoved repo not showed in repo list");
        }

        cd::Repository::remove(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to remove Repository")?;

        // Purge removed repository
        cd::Repository::purge(&ctx.client, &ctx.project.name, repo_name)
            .await
            .context("Failed to purge removed Repository")?;

        let removed_repos = cd::Repository::list_removed_by_project_name(&ctx.client, &ctx.project.name)
            .await
            .context("Failed to list removed repositories")?;

        let mut found = false;
        for repo in removed_repos.iter() {
            if repo.name == repo_name {
                found = true;
            }
        }
        if found {
            bail!("Purged repo showed in removed repo list");
        }


        Ok(())
    }.boxed()
}

#[cfg(test)]
#[tokio::test]
async fn test_repos() {
    run_test(t1).await
}
