use std::process::Command;

#[test]
fn test_cli_translate() {
    let output = Command::new("cargo")
        .args(["run", "-p", "epcis-cli", "--", "--translate", "urn:epc:id:sgtin:4012345.098765.12345"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "https://id.gs1.org/01/04012345987652/21/12345");
}

#[test]
fn test_cli_translate_dl() {
    let output = Command::new("cargo")
        .args(["run", "-p", "epcis-cli", "--", "--translate", "https://id.gs1.org/01/04012345987652/21/12345"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "urn:epc:id:sgtin:4012345.098765.12345");
}

#[test]
fn test_cli_hash_json() {
    let json_input = r#"{
      "@context": ["https://ref.gs1.org/standards/epcis/2.0.0/epcis-context.jsonld"],
      "type": "EPCISDocument",
      "schemaVersion": "2.0",
      "creationDate": "2020-03-04T11:00:30.000Z",
      "epcisBody": {
        "eventList": [
          {
            "type": "ObjectEvent",
            "eventTime": "2020-03-04T11:00:30.000Z",
            "eventTimeZoneOffset": "+00:00",
            "action": "OBSERVE",
            "epcList": ["urn:epc:id:sgtin:4012345.098765.12345"]
          }
        ]
      }
    }"#;

    use std::io::Write;
    let mut child = Command::new("cargo")
        .args(["run", "-p", "epcis-cli", "--", "-p"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(json_input.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");
    assert!(output.status.success(), "Stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("eventType=ObjectEvent"));
    assert!(stdout.contains("action=OBSERVE"));
}
