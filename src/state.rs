use tera::Tera;
use crate::services::blogpost_service::BlogPostService;
use futures::future;
use std::error::Error;

use crate::{
    models,
    services::{
        portfolio_service::PortfolioService,
    },
};

#[derive(Debug, Clone)]
pub struct AppState {
    portfolio_entries: Vec<models::PortfolioEntry>,
    tera: Tera,
}

impl AppState {
    pub async fn from_s3_buckets(
        client: &aws_sdk_s3::Client,
        portfolio_entries_bucket: &str,
        templates_bucket: &str,
        dev: bool
    ) -> Self {
        let portfolio_entry_objects = Self::aws_list_objects(client, portfolio_entries_bucket)
            .await
            .unwrap();
        let template_objects = Self::aws_list_objects(client, templates_bucket).await.unwrap();

        let portfolio_entries = PortfolioService::parse_many(
            future::join_all(
                portfolio_entry_objects
                    .iter()
                    .map(|object| Self::aws_get_object(client, portfolio_entries_bucket, object)),
            )
            .await
            .into_iter()
            .map(|result| result.unwrap()),
        );

        let mut tera = Tera::default();

        future::join_all(
            template_objects.iter().map(|object| Self::aws_get_object(client, templates_bucket, object)),
        ).await.into_iter().map(|result| result.unwrap()).zip(template_objects).for_each(|(contents, name)| tera.add_raw_template(&name, &contents).expect("ran into an error creating a template"));

        if dev {
            tera.add_template_file("templates/portfolio.tera.html", Some("portfolio.tera.html")).unwrap();
        }
        Self {portfolio_entries, tera}
    }

    pub fn get_portfolio_entries(&self) -> &Vec<models::PortfolioEntry> {
        &self.portfolio_entries
    }

    pub fn get_templater(&self) -> &Tera {
        &self.tera
    }

    pub async fn aws_list_objects(
        client: &aws_sdk_s3::Client,
        bucket: &str,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut response = client
            .list_objects_v2()
            .bucket(bucket.to_owned())
            .max_keys(10)
            .into_paginator()
            .send();

        let mut objects = Vec::new();
        while let Some(result) = response.next().await {
            match result {
                Ok(output) => {
                    for object in output.contents() {
                        objects.push(object.key().unwrap_or_default().to_string());
                    }
                }
                Err(err) => {
                    eprintln!("Error listing objects: {}", err);
                    return Err(Box::new(err));
                }
            }
        }

        Ok(objects)
    }

    pub async fn aws_get_object(
        client: &aws_sdk_s3::Client,
        bucket: &str,
        key: &str,
    ) -> Result<String, Box<dyn Error>> {
        let response = client
            .get_object()
            .bucket(bucket.to_owned())
            .key(key.to_owned())
            .send()
            .await?;

        let body = response.body.collect().await?;
        let content = String::from_utf8(body.into_bytes().to_vec())?;

        Ok(content)
    }
}
