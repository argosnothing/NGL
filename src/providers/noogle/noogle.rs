use super::schema::NoogleResponse;
use crate::providers::{Provider, noogle::schema::Doc};

pub struct Noogle {}
static ENDPOINT_URL: &str = "https://noogle.dev/api/v1/data";

impl Provider for Noogle {
    async fn pull_data() -> crate::schema::NGLResponse {
        let response = reqwest::get(ENDPOINT_URL)
            .await
            .expect("Failed to fetch from Noogle")
            .json::<NoogleResponse>()
            .await
            .expect("Failed to parse Noogle response");

        println!("Fetched {} docs", response.data.len());

        let with_sig = response
            .data
            .iter()
            .filter(|d| d.meta.signature.is_some())
            .count();
        println!("Docs with signatures: {}", with_sig);

        println!("\nFirst 10 docs WITH signatures:");
        for doc in response
            .data
            .iter()
            .filter(|d| d.meta.signature.is_some())
            .take(10)
        {
            println!("\n---");
            println!("Title: {}", doc.meta.title);
            println!("Signature: {:?}", doc.meta.signature);
            println!(
                "Content: {:?}",
                doc.content.as_ref().and_then(|c| c.content.as_ref())
            );
        }

        unreachable!()
    }

    fn get_name() -> String {
        "noogle".to_owned()
    }
}
