use centraldogma as cd;

#[cfg(test)]
#[tokio::test]
async fn test_projects() {
    let client = cd::Client::new_with_token("http://localhost:36462".to_string(), None)
        .await
        .unwrap();
    let projects = cd::Project::list(&client)
        .await
        .expect("Failed to list projects");
    assert_eq!(0, projects.len());

    let invalid_prj_name = "Test Project";
    let invalid_new_project = cd::Project::create(&client, invalid_prj_name).await;
    assert!(matches!(invalid_new_project, Err(_)));

    let prj_name = "TestProject";
    let new_project = cd::Project::create(&client, prj_name)
        .await
        .expect("Failed to create new project");
    assert_eq!(prj_name, new_project.name);

    let projects = cd::Project::list(&client)
        .await
        .expect("Failed to list projects");
    assert_eq!(1, projects.len());
    assert_eq!(prj_name, projects[0].name);

    cd::Project::remove(&client, prj_name)
        .await
        .expect("Failed to remove the project");

    let removed_projects = cd::Project::list_removed(&client)
        .await
        .expect("Failed to list removed projects");
    assert_eq!(1, removed_projects.len());
    assert_eq!(prj_name, removed_projects[0]);

    let unremove_project = cd::Project::unremove(&client, prj_name)
        .await
        .expect("Failed to unremove project");
    assert_eq!(prj_name, unremove_project.name);

    cd::Project::remove(&client, prj_name)
        .await
        .expect("Failed to remove the project again");

    cd::Project::purge(&client, prj_name)
        .await
        .expect("Failed to purge the project");
}
