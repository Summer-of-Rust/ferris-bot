use crate::configuration;
use crate::model::configurable::ConfigurableValue;


use std::io;
use std::io::Error;
use std::process::Command;
use std::process::Stdio;


/// Settings for our container
#[derive(Clone)]
pub struct ContainerSettings {
    pub cpu: String,
    pub memory: String,
    pub swap: String,
    pub image: String,
    pub max_runtime: u64,
}

pub trait ContainerActions {
    fn generate_runtime_flags(&self, is_container: bool) -> String;
    fn pull_image(&self) -> Result<(), Error>;
    fn invoke_command(&self, command: String) -> io::Result<std::process::Child>;
}

impl ContainerActions for ContainerSettings {
    /// Turns a ContainerSettings instance into a string of CLI args for Podman or Docker
    /// is_container: describes if we are running rustbot in a container
    fn generate_runtime_flags(&self, is_container: bool) -> String {
        // BUG: when swap is included, we get a OCI runtime error as memory+swap is greater than configured memory
        // fix and re-add swap constraint
        // NOTE: podman-in-podman requires cgroups to set resources, which isn't available within nested containers
        // so, admins will have to limit the resources on the outer container themselves
        if is_container {
            String::from("")
        } else {
            format!("--cpus={} --memory={}", self.cpu, self.memory)
        }
    }

    /// Pulls a container image from a registry
    fn pull_image(&self) -> Result<(), Error> {
        let output = Command::new("podman")
            .arg("pull")
            .arg(&self.image)
            .status()
            .expect("failed to execute process");

        let status = output.code().expect("No output code");

        if status == 0 {
            Ok(())
        } else {
            Result::Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Could not pull container image, got error code {}", status),
            ))
        }
    }

    fn invoke_command(&self, command: String) -> io::Result<std::process::Child> {
        let container_command = format!(
            "podman run --rm {} {} {}",
            self.generate_runtime_flags(configuration::IS_RUNNING_IN_CONTAINER.value()),
            self.image,
            command
        );

        println!("{}", container_command);

        // Because std::command does not give me the ability to override / modify
        // how arguments are escaped I have to do some stupid hack to make this
        // work. For example, if I wanted to run
        // podman run rustbot:latest ls -al
        // this would be impossible if I did
        //
        //  std::process::Command::new("podman")
        //    .args(["run", "rustbot:latest", "ls -al"])
        //    .output()
        //    .expect("failed to invoke container");
        //
        // As the ls -al would be quoted, and the container would try to execute
        // `ls -al` which would fail. The alternative is to seperate "ls", "-al"
        // which would also fail as the container would run `ls` then `-al`
        // ... what a stupid design
        // So instead of embracing the safety this API gives you, i'm just invoking
        // a shell with a payload I deem as safe
        Command::new("sh")
            .args(["-c", container_command.as_str()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

/// Gets the default container settings
pub fn get_container_settings() -> ContainerSettings {
    ContainerSettings {
        cpu: (*configuration::CONTAINER_CPU).value(),
        image: (*configuration::CONTAINER_IMAGE).value(),
        memory: (*configuration::CONTAINER_MEMORY).value(),
        swap: (*configuration::CONTAINER_SWAP).value(),
        max_runtime: (*configuration::CONTAINER_MAX_RUNTIME).value(),
    }
}
