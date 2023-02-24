use crate::tools::docker_runner::DockerRunner;

pub fn collect_mounts() {
    let mut docker = DockerRunner::new();

    docker.fetch_mount_dbfiles();
    docker.convert_dbfiles_into_csv();
}
