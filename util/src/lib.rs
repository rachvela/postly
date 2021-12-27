use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt;

#[derive(Debug)]
pub enum PostlyError {
  PostError,
  AccountError
}

impl std::error::Error for PostlyError {}

impl fmt::Display for PostlyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
        PostlyError::PostError => write!(f, "Post Error"),
        PostlyError::AccountError => write!(f, "Account Error"),
      }
    }
  }

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PostlyAccountIndex {
    pub post_n: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PostlyAccount {
    pub post: String,
}