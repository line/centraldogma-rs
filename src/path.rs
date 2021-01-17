const PATH_PREFIX: &str = "/api/v1";
const PROJECT_PATH: &str = "projects";
const REPO_PATH: &str = "repos";

pub(crate) fn projects_path() -> String {
    format!("{}/{}", PATH_PREFIX, PROJECT_PATH)
}

pub(crate) fn removed_projects_path() -> String {
    format!("{}/{}?status=removed", PATH_PREFIX, PROJECT_PATH)
}

pub(crate) fn project_path(project_name: &str) -> String {
    format!("{}/{}/{}", PATH_PREFIX, PROJECT_PATH, project_name)
}

pub(crate) fn removed_project_path(project_name: &str) -> String {
    format!("{}/{}/{}/removed", PATH_PREFIX, PROJECT_PATH, project_name)
}

pub(crate) fn repos_path(project_name: &str) -> String {
    format!(
        "{}/{}/{}/{}",
        PATH_PREFIX, PROJECT_PATH, project_name, REPO_PATH
    )
}

pub(crate) fn removed_repos_path(project_name: &str) -> String {
    format!(
        "{}/{}/{}/{}?status=removed",
        PATH_PREFIX, PROJECT_PATH, project_name, REPO_PATH
    )
}

pub(crate) fn repo_path(project_name: &str, repo_name: &str) -> String {
    format!(
        "{}/{}/{}/{}/{}",
        PATH_PREFIX, PROJECT_PATH, project_name, REPO_PATH, repo_name
    )
}

pub(crate) fn removed_repo_path(project_name: &str, repo_name: &str) -> String {
    format!(
        "{}/{}/{}/{}/{}/removed",
        PATH_PREFIX, PROJECT_PATH, project_name, REPO_PATH, repo_name
    )
}
