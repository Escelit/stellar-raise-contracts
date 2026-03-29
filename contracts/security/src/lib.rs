#![no_std]

pub mod multi_signature_execution;
pub mod output_sanitization;
pub mod security_testing_automation;
pub mod security_remediation;

#[cfg(test)]
#[path = "multi_signature_execution.test.rs"]
mod multi_signature_execution_test;

#[cfg(test)]
#[path = "output_sanitization.test.rs"]
mod output_sanitization_test;

#[cfg(test)]
#[path = "security_testing_automation.test.rs"]
mod security_testing_automation_test;
#[cfg(test)]
#[path = "security_remediation.test.rs"]
mod security_remediation_test;
