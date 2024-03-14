use std::fs::Permissions;
use std::path::Path;
use std::time::Duration;

#[cfg(unix)]
pub fn set_executable<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = path.as_ref().metadata()?.permissions();
    let new_permissions = Permissions::from_mode(permissions.mode() | 0o111);

    std::fs::set_permissions(path.as_ref(), new_permissions)
}

pub(crate) fn wait_for_http_200(url: &str) -> Result<Duration, ureq::Error> {
    let backoff =
        exponential_backoff::Backoff::new(32, Duration::from_millis(10), Duration::from_secs(60));

    let mut backoff_durations = backoff.into_iter();
    let mut backoff_acc = Duration::ZERO;

    loop {
        match ureq::get(url).call() {
            Ok(_) => return Ok(backoff_acc),
            Err(error) => match backoff_durations.next() {
                None => return Err(error),
                Some(backoff_duration) => {
                    std::thread::sleep(backoff_duration);
                    backoff_acc += backoff_duration;
                    continue;
                }
            },
        }
    }
}
