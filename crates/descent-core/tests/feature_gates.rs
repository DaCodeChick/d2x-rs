//! Tests to verify that feature gates work correctly

use descent_core::models::Model;

#[test]
fn test_pof_always_available() {
    // POF should always be available regardless of features
    // Just EOF opcode (minimal valid POF)
    let data = vec![0x00, 0x00];
    let result = Model::from_pof(&data);
    assert!(result.is_ok());
}

#[test]
#[cfg(feature = "hires-assets")]
fn test_ase_available_with_feature() {
    // ASE should be available when hires-assets feature is enabled
    let data = "*3DSMAX_ASCIIEXPORT 200\n*SCENE {\n}\n";
    let result = Model::from_ase(data);
    assert!(result.is_ok());
}

#[test]
fn test_auto_detect_pof() {
    // Auto-detection should work for POF
    // Just EOF opcode (minimal valid POF)
    let data = vec![0x00, 0x00];
    let result = Model::parse(data);
    assert!(result.is_ok());
    if let Ok(model) = result {
        assert_eq!(model.format_type(), "POF");
    }
}

#[test]
#[cfg(feature = "hires-assets")]
fn test_auto_detect_ase() {
    // Auto-detection should work for ASE when feature is enabled
    let data = b"*3DSMAX_ASCIIEXPORT 200\n*SCENE {\n}\n".to_vec();
    let result = Model::parse(data);
    assert!(result.is_ok());
    if let Ok(model) = result {
        assert_eq!(model.format_type(), "ASE");
    }
}
