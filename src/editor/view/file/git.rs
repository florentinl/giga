use gix;

pub trait Vcs {
    fn get_ref(&self) -> String;
}

pub struct Git {
    repo: gix::Repository,
}

impl Git {
    pub fn open() -> Option<Self> {
        if let Ok(repo) = gix::discover(".") {
            Some(Self { repo })
        } else {
            None
        }
    }
}

impl Vcs for Git {
    fn get_ref(&self) -> String {
        match self.repo.head_name().unwrap() {
            Some(name) => {
                let reference = name.to_string();
                // Remove the "refs/heads/" prefix
                reference.trim_start_matches("refs/heads/").to_string()
            }
            None => {
                // No branch, it is a detached head
                let commit = self.repo.head_commit().unwrap();
                commit.short_id().unwrap().to_string()
            }
        }
    }
}
