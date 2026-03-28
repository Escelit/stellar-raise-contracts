#![no_std]

pub mod multi_signature_execution;
pub mod output_sanitization;
pub mod security_testing_automation;

#[cfg(test)]
#[path = "multi_signature_execution.test.rs"]
mod multi_signature_execution_test;

#[cfg(test)]
#[path = "output_sanitization.test.rs"]
mod output_sanitization_test;

#[cfg(test)]
#[path = "security_testing_automation.test.rs"]
mod security_testing_automation_test;
