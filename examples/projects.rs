use centraldogma as cd;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = cd::Client::new_with_token("http://localhost:36462".to_string(), None).await?;
    let projects = cd::Project::list(&client).await?;
    println!("Project list: {:?}", projects);

    let prj_name = "Test3";
    let new_project = cd::Project::create(&client, prj_name).await;
    println!("Create New Project: {:?}", new_project);

    let projects = cd::Project::list(&client).await?;
    println!("Project list: {:?}", projects);

    let remove_project = cd::Project::remove(&client, prj_name).await;
    println!("Remove project: {}, result: {:?}", prj_name, remove_project);

    let removed_projects = cd::Project::list_removed(&client).await?;
    println!("Removed Project list: {:?}", removed_projects);

    let unremove_project = cd::Project::unremove(&client, prj_name).await;
    println!("Unremove project: {}, result: {:?}", prj_name, unremove_project);

    let remove_project = cd::Project::remove(&client, prj_name).await;
    println!("Remove project again: {}, result: {:?}", prj_name, remove_project);

    let purge_project = cd::Project::purge(&client, prj_name).await;
    println!("Purge project: {}, result: {:?}", prj_name, purge_project);

    Ok(())

}
