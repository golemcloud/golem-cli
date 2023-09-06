use async_trait::async_trait;
use clap::Subcommand;
use clap::builder::ValueParser;
use golem_client::model::InvokeParameters;
use crate::clients::CloudAuthentication;
use crate::clients::instance::InstanceClient;
use crate::template::TemplateHandler;
use crate::model::{TemplateIdOrName, GolemError, GolemResult, InstanceName, InvocationKey, JsonValueParser};
use crate::parse_key_val;

#[derive(Subcommand, Debug)]
#[command()]
pub enum InstanceSubcommand {
    #[command()]
    Add {
        #[command(flatten)]
        template_id_or_name: TemplateIdOrName,

        #[arg(short, long)]
        instance_name: InstanceName,

        #[arg(short, long, value_parser = parse_key_val)]
        env: Vec<(String, String)>,

        #[arg(value_name = "args")]
        args: Vec<String>,
    },
    #[command()]
    InvocationKey {
        #[command(flatten)]
        template_id_or_name: TemplateIdOrName,

        #[arg(short, long)]
        instance_name: InstanceName,
    },
    #[command()]
    InvokeAndAwait {
        #[command(flatten)]
        template_id_or_name: TemplateIdOrName,

        #[arg(short, long)]
        instance_name: InstanceName,

        #[arg(short = 'k', long)]
        invocation_key: Option<InvocationKey>,

        #[arg(short, long)]
        function: String,

        #[arg(short = 'j', long, value_name = "json", value_parser = ValueParser::new(JsonValueParser))]
        parameters: serde_json::value::Value,

        #[arg(short = 's', long, default_value_t = false)]
        use_stdio: bool,
    },
    #[command()]
    Invoke {
        #[command(flatten)]
        template_id_or_name: TemplateIdOrName,

        #[arg(short, long)]
        instance_name: InstanceName,

        #[arg(short, long)]
        function: String,

        #[arg(short = 'j', long, value_name = "json", value_parser = ValueParser::new(JsonValueParser))]
        parameters: serde_json::value::Value,
    },
    #[command()]
    Connect {
        #[command(flatten)]
        template_id_or_name: TemplateIdOrName,

        #[arg(short, long)]
        instance_name: InstanceName,
    },
    #[command()]
    Interrupt {
        #[command(flatten)]
        template_id_or_name: TemplateIdOrName,

        #[arg(short, long)]
        instance_name: InstanceName,
    },
    #[command()]
    SimulatedCrash {
        #[command(flatten)]
        template_id_or_name: TemplateIdOrName,

        #[arg(short, long)]
        instance_name: InstanceName,
    },
    #[command()]
    Delete {
        #[command(flatten)]
        template_id_or_name: TemplateIdOrName,

        #[arg(short, long)]
        instance_name: InstanceName,
    },
    #[command()]
    Get {
        #[command(flatten)]
        template_id_or_name: TemplateIdOrName,

        #[arg(short, long)]
        instance_name: InstanceName,
    },
}

#[async_trait]
pub trait InstanceHandler {
    async fn handle(&self, auth: &CloudAuthentication, subcommand: InstanceSubcommand) -> Result<GolemResult, GolemError>;
}

pub struct InstanceHandlerLive<'r, C: InstanceClient + Send + Sync, R: TemplateHandler + Send + Sync> {
    pub client:C,
    pub templates: &'r R
}

#[async_trait]
impl <'r, C: InstanceClient + Send + Sync, R: TemplateHandler + Send + Sync> InstanceHandler for InstanceHandlerLive<'r, C, R> {
    async fn handle(&self, auth: &CloudAuthentication, subcommand: InstanceSubcommand) -> Result<GolemResult, GolemError> {
        match subcommand {
            InstanceSubcommand::Add { template_id_or_name, instance_name, env, args  } => {
                let template_id = self.templates.resolve_id(template_id_or_name, &auth).await?;

                let inst = self.client.new_instance(instance_name, template_id, args, env, &auth).await?;

                Ok(GolemResult::Ok(Box::new(inst)))
            }
            InstanceSubcommand::InvocationKey { template_id_or_name, instance_name } => {
                let template_id = self.templates.resolve_id(template_id_or_name, &auth).await?;

                let key = self.client.get_invocation_key(&instance_name, &template_id, &auth).await?;

                Ok(GolemResult::Ok(Box::new(key)))
            }
            InstanceSubcommand::InvokeAndAwait { template_id_or_name, instance_name,  invocation_key, function, parameters, use_stdio  } => {
                let template_id = self.templates.resolve_id(template_id_or_name, &auth).await?;

                let invocation_key = match invocation_key {
                    None => self.client.get_invocation_key(&instance_name, &template_id, auth).await?,
                    Some(key) => key,
                };
                
                let res = self.client.invoke_and_await(instance_name, template_id, function, InvokeParameters{params: parameters}, invocation_key, use_stdio, auth).await?;

                Ok(GolemResult::Json(res.result))
            }
            InstanceSubcommand::Invoke { template_id_or_name, instance_name, function, parameters } => {
                let template_id = self.templates.resolve_id(template_id_or_name, &auth).await?;

                self.client.invoke(instance_name, template_id, function, InvokeParameters{params: parameters}, auth).await?;

                Ok(GolemResult::Str("Invoked".to_string()))
            }
            InstanceSubcommand::Connect { template_id_or_name, instance_name } => {
                let template_id = self.templates.resolve_id(template_id_or_name, &auth).await?;

                self.client.connect(instance_name, template_id, auth).await?;

                Err(GolemError("connect should never complete".to_string()))
            }
            InstanceSubcommand::Interrupt { template_id_or_name, instance_name } => {
                let template_id = self.templates.resolve_id(template_id_or_name, &auth).await?;

                self.client.interrupt(instance_name, template_id, auth).await?;

                Ok(GolemResult::Str("Interrupted".to_string()))
            }
            InstanceSubcommand::SimulatedCrash { template_id_or_name, instance_name } => {
                let template_id = self.templates.resolve_id(template_id_or_name, &auth).await?;

                self.client.simulated_crash(instance_name, template_id, auth).await?;

                Ok(GolemResult::Str("Done".to_string()))
            }
            InstanceSubcommand::Delete { template_id_or_name, instance_name } => {
                let template_id = self.templates.resolve_id(template_id_or_name, &auth).await?;

                self.client.delete(instance_name, template_id, auth).await?;

                Ok(GolemResult::Str("Deleted".to_string()))
            }
            InstanceSubcommand::Get { template_id_or_name, instance_name } => {
                let template_id = self.templates.resolve_id(template_id_or_name, &auth).await?;

                let mata = self.client.get_metadata(instance_name, template_id, auth).await?;

                Ok(GolemResult::Ok(Box::new(mata)))
            }
        }
    }
}