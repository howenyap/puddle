use crate::app::CliApp;
use clap::Subcommand;
use puddle::models::user::User;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub(crate) enum UserCommand {
    #[command(about = "Show the authenticated user")]
    Me,
}

impl CliApp {
    pub(crate) async fn run_user(
        &self,
        command: UserCommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            UserCommand::Me => self.user_me().await,
        }
    }

    async fn user_me(&self) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.user().me().await?;
        println!("{}", UserDisplay(&response.data));
        Ok(())
    }
}

struct UserDisplay<'a>(&'a User);

impl Display for UserDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let user = self.0;
        writeln!(f, "id: {}", user.id)?;
        if let Some(full_name) = user.full_name.as_deref() {
            writeln!(f, "full_name: {full_name}")?;
        }
        if let Some(email) = user.email.as_deref() {
            write!(f, "email: {email}")?;
        }

        Ok(())
    }
}
