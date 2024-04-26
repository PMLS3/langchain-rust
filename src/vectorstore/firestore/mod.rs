use firestore_db_and_auth::{documents::Document as FirestoreDocument, sessions::ServiceSession, credentials::Credentials, errors::FirestoreError};
use std::error::Error;
use async_trait::async_trait;
use crate::vectorstore::{VectorStore, VecStoreOptions};
use crate::models::VectorDocument;

pub struct FirestoreStore {
    session: ServiceSession,
    collection_name: String,
}

impl FirestoreStore {
    pub async fn new(credentials_path: &str, collection_name: &str) -> Result<Self, FirestoreError> {
        let credentials = Credentials::from_file(credentials_path).await?;
        let session = ServiceSession::new(credentials)?;
        Ok(FirestoreStore {
            session,
            collection_name: collection_name.to_string(),
        })
    }
}

#[async_trait]
impl VectorStore for FirestoreStore {
    async fn add_documents(&self, docs: &[VectorDocument], opt: &VecStoreOptions) -> Result<Vec<String>, Box<dyn Error>> {
        let mut added_doc_ids = Vec::new();
        for doc in docs {
            let firestore_doc = FirestoreDocument::from_serde(&doc)?;
            let result = self.session.create(&self.collection_name, &firestore_doc, opt.write_concern).await?;
            added_doc_ids.push(result.name);
        }
        Ok(added_doc_ids)
    }

    async fn similarity_search(&self, query_vector: &[f32], top_k: usize, opt: &VecStoreOptions) -> Result<Vec<VectorDocument>, Box<dyn Error>> {
        let query = FirestoreDocument::from_serde(query_vector)?;
        let documents = self.session.query(&self.collection_name, &query, top_k).await?;

        let mut results = Vec::new();
        for doc in documents {
            let vec_doc: VectorDocument = serde_json::from_str(&doc.fields_json)?;
            results.push(vec_doc);
        }
        Ok(results)
    }
}
