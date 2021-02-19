use centraldogma as cd;

#[cfg(test)]
#[tokio::test]
async fn test_projects() {
    let client = cd::Client::from_token("http://localhost:36462", None)
        .await
        .unwrap();
    let projects = cd::project::list(&client)
        .await
        .expect("Failed to list projects");
    assert_eq!(0, projects.len());

    let invalid_prj_name = "Test Project";
    let invalid_new_project = cd::project::create(&client, invalid_prj_name).await;
    assert!(matches!(invalid_new_project, Err(_)));

    let prj_name = "TestProject";
    let new_project = cd::project::create(&client, prj_name)
        .await
        .expect("Failed to create new project");
    assert_eq!(prj_name, new_project.name);

    let projects = cd::project::list(&client)
        .await
        .expect("Failed to list projects");
    assert_eq!(1, projects.len());
    assert_eq!(prj_name, projects[0].name);

    cd::project::remove(&client, prj_name)
        .await
        .expect("Failed to remove the project");

    let removed_projects = cd::project::list_removed(&client)
        .await
        .expect("Failed to list removed projects");
    assert_eq!(1, removed_projects.len());
    assert_eq!(prj_name, removed_projects[0]);

    let unremove_project = cd::project::unremove(&client, prj_name)
        .await
        .expect("Failed to unremove project");
    assert_eq!(prj_name, unremove_project.name);

    cd::project::remove(&client, prj_name)
        .await
        .expect("Failed to remove the project again");

    cd::project::purge(&client, prj_name)
        .await
        .expect("Failed to purge the project");
}
