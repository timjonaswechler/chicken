#!/usr/bin/env python3
import argparse
import os
import subprocess
import sys

CRATE_CONFIG = {
    "chicken_states": {"features": {"hosted": "hosted", "headless": "headless"}},
    "chicken_network": {
        "features": {
            "default": "",
            "server": "server",
            "client": "client",
            "all": "server,client",
        },
        "test_threads": True,
    },
    "chicken_protocols": {
        "features": {
            "default": "",
            "server": "server",
            "client": "client",
            "all": "server,client",
        },
        "test_threads": False,
    },
    "chicken_settings": {
        "features": {
            "default": "",
        },
        "test_threads": False,
    },
    "chicken": {
        "features": {
            "default": "",
        },
        "test_threads": False,
    },
}

ALL_CRATES = list(CRATE_CONFIG.keys())


def get_features_for_crate(crate: str, feature_arg: str | None) -> list[str]:
    if crate not in CRATE_CONFIG:
        print(f"Warning: {crate} not in config, testing without features")
        return [""]

    config = CRATE_CONFIG[crate]
    features_map = config["features"]

    if feature_arg is None:
        return list(features_map.values())

    if feature_arg in features_map:
        return [features_map[feature_arg]]

    return [feature_arg]


def test_crate(crate: str, features: str, test_threads: bool = False) -> int:
    cmd = ["cargo", "test", "-p", crate]
    if features:
        cmd.extend(["--features", features])

    if test_threads:
        cmd.extend(["--", "--test-threads=1"])

    env = os.environ.copy()
    env["CARGO_TERM_COLOR"] = "always"

    print(f"\n{'=' * 50}")
    print(f"Testing {crate}" + (f" with features: {features}" if features else ""))
    print(f"Command: {' '.join(cmd)}")
    print(f"{'=' * 50}\n")

    result = subprocess.run(cmd, env=env)
    return result.returncode


def main():
    parser = argparse.ArgumentParser(description="Run cargo tests for chicken crates")
    parser.add_argument(
        "-c",
        "--crate",
        help=f"Crate to test (default: all). Available: {', '.join(ALL_CRATES)}",
    )
    parser.add_argument(
        "-f",
        "--features",
        help="Features to test. If not provided, tests all known feature sets. Can be 'default', 'server', 'client', 'all', or custom comma-separated features",
    )
    parser.add_argument(
        "--list", action="store_true", help="List available crates and exit"
    )

    args = parser.parse_args()

    if args.list:
        print("Available crates:")
        for crate, config in CRATE_CONFIG.items():
            features = list(config["features"].keys())
            print(f"  {crate}: features={features}")
        return 0

    crates_to_test = [args.crate] if args.crate else ALL_CRATES

    failed = []
    for crate in crates_to_test:
        features_list = get_features_for_crate(crate, args.features)

        for features in features_list:
            test_threads = CRATE_CONFIG.get(crate, {}).get("test_threads", False)
            returncode = test_crate(crate, features, test_threads)

            if returncode != 0:
                print(
                    f"FAILED: {crate}"
                    + (f" (features: {features})" if features else "")
                )
                failed.append((crate, features))

    if failed:
        print("\n" + "=" * 50)
        print("FAILED TESTS:")
        for crate, features in failed:
            print(f"  {crate}" + (f" (features: {features})" if features else ""))
        print("=" * 50)
        return 1

    print("\n" + "=" * 50)
    print("ALL TESTS PASSED")
    print("=" * 50)
    return 0


if __name__ == "__main__":
    sys.exit(main())
