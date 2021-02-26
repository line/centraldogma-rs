use cd::ProjectService;
use centraldogma as cd;

#[cfg(test)]
#[tokio::test]
async fn test_projects() {
    let client = cd::Client::new("http://localhost:36462", None)
        .await
        .unwrap();
    let projects = client
        .list_projects()
        .await
        .expect("Failed to list projects");
    assert_eq!(0, projects.len());

    let invalid_prj_name = "Test Project";
    let invalid_new_project = client.create_project(invalid_prj_name).await;
    assert!(matches!(invalid_new_project, Err(_)));

    let prj_name = "TestProject";
    let new_project = client
        .create_project(prj_name)
        .await
        .expect("Failed to create new project");
    assert_eq!(prj_name, new_project.name);

    let projects = client
        .list_projects()
        .await
        .expect("Failed to list projects");
    assert_eq!(1, projects.len());
    assert_eq!(prj_name, projects[0].name);

    client
        .remove_project(prj_name)
        .await
        .expect("Failed to remove the project");

    let removed_projects = client
        .list_removed_projects()
        .await
        .expect("Failed to list removed projects");
    assert_eq!(1, removed_projects.len());
    assert_eq!(prj_name, removed_projects[0]);

    let unremove_project = client
        .unremove_project(prj_name)
        .await
        .expect("Failed to unremove project");
    assert_eq!(prj_name, unremove_project.name);

    client
        .remove_project(prj_name)
        .await
        .expect("Failed to remove the project again");

    client
        .purge_project(prj_name)
        .await
        .expect("Failed to purge the project");
}
