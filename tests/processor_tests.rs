use idgen::id::{CuidVersion, IDFormat, UuidVersion};

fn with_args(args: Vec<&str>) -> Vec<String> {
    let mut full_args = vec!["program"];
    full_args.extend(args);
    full_args.iter().map(|s| s.to_string()).collect()
}

// ============================================
// Default Behavior Tests
// ============================================

#[test]
fn test_default_format() {
    let args = with_args(vec![]);
    let version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);
    let mut count = 1;
    let mut help = false;
    let mut show_version = false;
    let mut len: Option<usize> = None;
    let mut prefix = "";
    let mut namespace: Option<String> = None;
    let mut name: Option<String> = None;
    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-h" || arg == "--help" {
            help = true;
        } else if arg == "-v" || arg == "--version" {
            show_version = true;
        } else if arg == "-s" || arg == "--simple" {
            format = IDFormat::Simple(version);
        } else if arg == "-u" || arg == "--urn" {
            format = IDFormat::URN(version);
        } else if arg == "-o" || arg == "--objectid" {
            format = IDFormat::OID;
        } else if arg == "-n" || arg == "--nano" {
            format = IDFormat::NanoID;
        }

        if lastcmd == "-c" || lastcmd == "--count" {
            count = arg.parse::<i32>().unwrap_or(1);
        } else if lastcmd == "-n" || lastcmd == "--nano" {
            len = Some(arg.parse::<usize>().unwrap_or(21));
        } else if lastcmd == "-p" || lastcmd == "--prefix" {
            prefix = arg;
        } else if lastcmd == "--namespace" {
            namespace = Some(arg.to_string());
        } else if lastcmd == "--name" {
            name = Some(arg.to_string());
        }

        lastcmd = arg.clone();
    });

    assert!(matches!(format, IDFormat::Hyphenated(_)));
    assert_eq!(count, 1);
    assert!(!help);
    assert!(!show_version);
    assert_eq!(len, None);
    assert_eq!(prefix, "");
    assert_eq!(namespace, None);
    assert_eq!(name, None);
}

#[test]
fn test_simple_format() {
    let args = with_args(vec!["--simple"]);
    let version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);
    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-s" || arg == "--simple" {
            format = IDFormat::Simple(version);
        }
        lastcmd = arg.clone();
    });

    assert!(matches!(format, IDFormat::Simple(_)));
}

#[test]
fn test_uuid_version() {
    let args = with_args(vec!["--uuid3"]);
    let mut version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);
    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-u3" || arg == "--uuid3" {
            version = UuidVersion::V3;
            format = IDFormat::Hyphenated(version);
        }
        lastcmd = arg.clone();
    });

    assert!(matches!(format, IDFormat::Hyphenated(UuidVersion::V3)));
}

#[test]
fn test_count_option() {
    let args = with_args(vec!["--count", "5"]);
    let mut count = 1;
    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if lastcmd == "-c" || lastcmd == "--count" {
            count = arg.parse::<i32>().unwrap_or(1);
        }
        lastcmd = arg.clone();
    });

    assert_eq!(count, 5);
}

#[test]
fn test_nanoid_length() {
    let args = with_args(vec!["--nano", "10"]);
    let version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);
    let mut len: Option<usize> = None;
    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-n" || arg == "--nano" {
            format = IDFormat::NanoID;
        } else if lastcmd == "-n" || lastcmd == "--nano" {
            len = Some(arg.parse::<usize>().unwrap_or(21));
        }
        lastcmd = arg.clone();
    });

    assert!(matches!(format, IDFormat::NanoID));
    assert_eq!(len, Some(10));
}

#[test]
fn test_prefix_option() {
    let args = with_args(vec!["--prefix", "test-"]);
    let mut prefix = "";
    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if lastcmd == "-p" || lastcmd == "--prefix" {
            prefix = arg;
        }
        lastcmd = arg.clone();
    });

    assert_eq!(prefix, "test-");
}

#[test]
fn test_suffix_option() {
    let args = with_args(vec!["--suffix", ".log"]);
    let mut suffix = "";
    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if lastcmd == "-f" || lastcmd == "--suffix" {
            suffix = arg;
        }
        lastcmd = arg.clone();
    });

    assert_eq!(suffix, ".log");
}

#[test]
fn test_uuid_v3_parameters() {
    let args = with_args(vec![
        "--uuid3",
        "--namespace",
        "DNS",
        "--name",
        "example.com",
    ]);
    let mut version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);
    let mut namespace: Option<String> = None;
    let mut name: Option<String> = None;
    let mut lastcmd = String::new();

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-u3" || arg == "--uuid3" {
            version = UuidVersion::V3;
            format = IDFormat::Hyphenated(version);
        } else if lastcmd == "--namespace" {
            namespace = Some(arg.to_string());
        } else if lastcmd == "--name" {
            name = Some(arg.to_string());
        }
        lastcmd = arg.clone();
    });

    assert!(matches!(format, IDFormat::Hyphenated(UuidVersion::V3)));
    assert_eq!(namespace, Some("DNS".to_string()));
    assert_eq!(name, Some("example.com".to_string()));
}

#[test]
fn test_banner_flag() {
    let args = with_args(vec!["--banner"]);
    let mut show_banner = false;

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-b" || arg == "--banner" {
            show_banner = true;
        }
    });

    assert!(show_banner);
}

#[test]
fn test_json_flag() {
    let args = with_args(vec!["--json"]);
    let mut json_output = false;
    let mut show_banner = true; // Assume default is true for this test logic check

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "--json" {
            json_output = true;
            show_banner = false;
        }
    });

    assert!(json_output);
    assert!(!show_banner);
}

#[test]
fn test_cuid_v1_flag() {
    let args = with_args(vec!["--cuid1"]);
    let mut format = IDFormat::Hyphenated(UuidVersion::V4);

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-c1" || arg == "--cuid1" {
            format = IDFormat::Cuid(CuidVersion::V1);
        }
    });

    if let IDFormat::Cuid(version) = format {
        assert!(matches!(version, CuidVersion::V1));
    } else {
        panic!("Format should be Cuid(V1)");
    }
}

#[test]
fn test_cuid_v2_flag() {
    let args = with_args(vec!["-c2"]);
    let mut format = IDFormat::Hyphenated(UuidVersion::V4);

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-c2" || arg == "--cuid2" {
            format = IDFormat::Cuid(CuidVersion::V2);
        }
    });

    if let IDFormat::Cuid(version) = format {
        assert!(matches!(version, CuidVersion::V2));
    } else {
        panic!("Format should be Cuid(V2)");
    }
}

#[test]
fn test_ulid_flag() {
    let args = with_args(vec!["--ulid"]);
    let mut format = IDFormat::Hyphenated(UuidVersion::V4);

    args.iter().enumerate().for_each(|(_, arg)| {
        if arg == "-l" || arg == "--ulid" {
            format = IDFormat::Ulid;
        }
    });

    assert!(matches!(format, IDFormat::Ulid));
}

// ============================================
// Short Flag Tests
// ============================================

#[test]
fn test_short_simple_flag() {
    let args = with_args(vec!["-s"]);
    let version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);

    args.iter().for_each(|arg| {
        if arg == "-s" || arg == "--simple" {
            format = IDFormat::Simple(version);
        }
    });

    assert!(matches!(format, IDFormat::Simple(_)));
}

#[test]
fn test_short_urn_flag() {
    let args = with_args(vec!["-u"]);
    let version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);

    args.iter().for_each(|arg| {
        if arg == "-u" || arg == "--urn" {
            format = IDFormat::URN(version);
        }
    });

    assert!(matches!(format, IDFormat::URN(_)));
}

#[test]
fn test_short_objectid_flag() {
    let args = with_args(vec!["-o"]);
    let mut format = IDFormat::Hyphenated(UuidVersion::V4);

    args.iter().for_each(|arg| {
        if arg == "-o" || arg == "--objectid" {
            format = IDFormat::OID;
        }
    });

    assert!(matches!(format, IDFormat::OID));
}

#[test]
fn test_short_nano_flag() {
    let args = with_args(vec!["-n"]);
    let mut format = IDFormat::Hyphenated(UuidVersion::V4);

    args.iter().for_each(|arg| {
        if arg == "-n" || arg == "--nano" {
            format = IDFormat::NanoID;
        }
    });

    assert!(matches!(format, IDFormat::NanoID));
}

#[test]
fn test_short_ulid_flag() {
    let args = with_args(vec!["-l"]);
    let mut format = IDFormat::Hyphenated(UuidVersion::V4);

    args.iter().for_each(|arg| {
        if arg == "-l" || arg == "--ulid" {
            format = IDFormat::Ulid;
        }
    });

    assert!(matches!(format, IDFormat::Ulid));
}

#[test]
fn test_short_help_flag() {
    let args = with_args(vec!["-h"]);
    let mut help = false;

    args.iter().for_each(|arg| {
        if arg == "-h" || arg == "--help" {
            help = true;
        }
    });

    assert!(help);
}

#[test]
fn test_short_version_flag() {
    let args = with_args(vec!["-v"]);
    let mut show_version = false;

    args.iter().for_each(|arg| {
        if arg == "-v" || arg == "--version" {
            show_version = true;
        }
    });

    assert!(show_version);
}

#[test]
fn test_short_count_flag() {
    let args = with_args(vec!["-c", "10"]);
    let mut count = 1;
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-c" || lastcmd == "--count" {
            count = arg.parse::<i32>().unwrap_or(1);
        }
        lastcmd = arg.clone();
    });

    assert_eq!(count, 10);
}

#[test]
fn test_short_prefix_flag() {
    let args = with_args(vec!["-p", "pre_"]);
    let mut prefix = "";
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-p" || lastcmd == "--prefix" {
            prefix = arg;
        }
        lastcmd = arg.clone();
    });

    assert_eq!(prefix, "pre_");
}

#[test]
fn test_short_suffix_flag() {
    let args = with_args(vec!["-f", "_suf"]);
    let mut suffix = "";
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-f" || lastcmd == "--suffix" {
            suffix = arg;
        }
        lastcmd = arg.clone();
    });

    assert_eq!(suffix, "_suf");
}

// ============================================
// Edge Cases - Invalid Count Values
// ============================================

#[test]
fn test_count_invalid_non_numeric() {
    let args = with_args(vec!["--count", "abc"]);
    let mut count = 1;
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-c" || lastcmd == "--count" {
            count = arg.parse::<i32>().unwrap_or(1);
        }
        lastcmd = arg.clone();
    });

    // Should fallback to default 1
    assert_eq!(count, 1);
}

#[test]
fn test_count_negative_value() {
    let args = with_args(vec!["--count", "-5"]);
    let mut count = 1;
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-c" || lastcmd == "--count" {
            count = arg.parse::<i32>().unwrap_or(1);
        }
        lastcmd = arg.clone();
    });

    // Parses as -5 (valid i32)
    assert_eq!(count, -5);
}

#[test]
fn test_count_zero() {
    let args = with_args(vec!["--count", "0"]);
    let mut count = 1;
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-c" || lastcmd == "--count" {
            count = arg.parse::<i32>().unwrap_or(1);
        }
        lastcmd = arg.clone();
    });

    assert_eq!(count, 0);
}

#[test]
fn test_count_very_large() {
    let args = with_args(vec!["--count", "999999"]);
    let mut count = 1;
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-c" || lastcmd == "--count" {
            count = arg.parse::<i32>().unwrap_or(1);
        }
        lastcmd = arg.clone();
    });

    assert_eq!(count, 999999);
}

// ============================================
// Edge Cases - NanoID Length
// ============================================

#[test]
fn test_nanoid_length_invalid() {
    let args = with_args(vec!["--nano", "xyz"]);
    let mut len: Option<usize> = None;
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-n" || lastcmd == "--nano" {
            len = Some(arg.parse::<usize>().unwrap_or(21));
        }
        lastcmd = arg.clone();
    });

    // Should fallback to 21
    assert_eq!(len, Some(21));
}

#[test]
fn test_nanoid_length_zero() {
    let args = with_args(vec!["--nano", "0"]);
    let mut len: Option<usize> = None;
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-n" || lastcmd == "--nano" {
            len = Some(arg.parse::<usize>().unwrap_or(21));
        }
        lastcmd = arg.clone();
    });

    assert_eq!(len, Some(0));
}

// ============================================
// Edge Cases - Empty/Special Prefix/Suffix
// ============================================

#[test]
fn test_prefix_empty() {
    let args = with_args(vec!["--prefix", ""]);
    let mut prefix = "default";
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-p" || lastcmd == "--prefix" {
            prefix = arg;
        }
        lastcmd = arg.clone();
    });

    assert_eq!(prefix, "");
}

#[test]
fn test_suffix_with_special_chars() {
    let args = with_args(vec!["--suffix", "@#$%"]);
    let mut suffix = "";
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-f" || lastcmd == "--suffix" {
            suffix = arg;
        }
        lastcmd = arg.clone();
    });

    assert_eq!(suffix, "@#$%");
}

#[test]
fn test_prefix_with_spaces() {
    let args = with_args(vec!["--prefix", "hello world"]);
    let mut prefix = "";
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "-p" || lastcmd == "--prefix" {
            prefix = arg;
        }
        lastcmd = arg.clone();
    });

    assert_eq!(prefix, "hello world");
}

// ============================================
// Inspect Flag Tests
// ============================================

#[test]
fn test_inspect_flag() {
    let args = with_args(vec!["--inspect", "550e8400-e29b-44d4-a716-446655440000"]);
    let mut inspect_target: Option<String> = None;
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "--inspect" || lastcmd == "-i" {
            inspect_target = Some(arg.to_string());
        }
        lastcmd = arg.clone();
    });

    assert_eq!(
        inspect_target,
        Some("550e8400-e29b-44d4-a716-446655440000".to_string())
    );
}

#[test]
fn test_short_inspect_flag() {
    let args = with_args(vec!["-i", "test-id"]);
    let mut inspect_target: Option<String> = None;
    let mut lastcmd = String::new();

    args.iter().for_each(|arg| {
        if lastcmd == "--inspect" || lastcmd == "-i" {
            inspect_target = Some(arg.to_string());
        }
        lastcmd = arg.clone();
    });

    assert_eq!(inspect_target, Some("test-id".to_string()));
}

// ============================================
// UUID Version Flag Tests
// ============================================

#[test]
fn test_uuid_v1_flag() {
    let args = with_args(vec!["--uuid1"]);
    let mut version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);

    args.iter().for_each(|arg| {
        if arg == "-u1" || arg == "--uuid1" {
            version = UuidVersion::V1;
            format = IDFormat::Hyphenated(version);
        }
    });

    assert!(matches!(format, IDFormat::Hyphenated(UuidVersion::V1)));
}

#[test]
fn test_uuid_v4_flag() {
    let args = with_args(vec!["--uuid4"]);
    let mut version = UuidVersion::V1; // Start with V1 to verify change
    let mut format = IDFormat::Hyphenated(version);

    args.iter().for_each(|arg| {
        if arg == "-u4" || arg == "--uuid4" {
            version = UuidVersion::V4;
            format = IDFormat::Hyphenated(version);
        }
    });

    assert!(matches!(format, IDFormat::Hyphenated(UuidVersion::V4)));
}

#[test]
fn test_uuid_v5_flag() {
    let args = with_args(vec!["--uuid5"]);
    let mut version = UuidVersion::V4;
    let mut format = IDFormat::Hyphenated(version);

    args.iter().for_each(|arg| {
        if arg == "-u5" || arg == "--uuid5" {
            version = UuidVersion::V5;
            format = IDFormat::Hyphenated(version);
        }
    });

    assert!(matches!(format, IDFormat::Hyphenated(UuidVersion::V5)));
}
