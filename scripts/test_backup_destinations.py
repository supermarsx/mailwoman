import importlib.util
import contextlib
import io
import sys
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("destinations", Path(__file__).with_name("backup-destinations.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class Destinations(unittest.TestCase):
    def test_destination_filesystem_or_config_failure_does_not_skip_other_copies(self):
        for error in [PermissionError("secret-token-do-not-print"),
                      module.configparser.Error("secret-token-do-not-print")]:
            with self.subTest(error=type(error).__name__), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = root / "backup.tar.age"
                source.write_bytes(b"retained encrypted source")
                source.with_name("backup.release.env").write_text("IMAGE_ID=sha256:fixture\n")
                first, second = root / "first", root / "second"
                first.mkdir(); second.mkdir()
                config = root / "targets.json"
                config.write_text(json.dumps({"version": 1, "destinations": [
                    {"id": "first", "type": "local", "path": str(first)},
                    {"id": "second", "type": "local", "path": str(second)},
                ]}))
                preflight = module.preflight
                def refuse_first(row):
                    if row.get("id") == "first":
                        raise error
                    return preflight(row)
                stderr, stdout = io.StringIO(), io.StringIO()
                with patch.object(module, "preflight", side_effect=refuse_first), \
                     patch.object(sys, "argv", ["backup-destinations.py", "publish", "--config", str(config), "--source", str(source)]), \
                     contextlib.redirect_stderr(stderr), contextlib.redirect_stdout(stdout):
                    status = module.main()
                self.assertEqual(status, 1)
                self.assertEqual(source.read_bytes(), b"retained encrypted source")
                self.assertFalse((first / (source.name + ".complete.json")).exists())
                self.assertEqual((second / source.name).read_bytes(), source.read_bytes())
                self.assertTrue((second / (source.name + ".complete.json")).is_file())
                self.assertIn("second: publish complete", stdout.getvalue())
                self.assertIn(type(error).__name__, stderr.getvalue())
                self.assertNotIn("secret-token-do-not-print", stderr.getvalue() + stdout.getvalue())

    def test_invalid_root_and_disabled_targets(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / "targets.json"
            for invalid in [[], None, 7, "invalid"]:
                path.write_text(json.dumps(invalid))
                with self.assertRaises(module.BackupError):
                    module.load_config(path)
            path.write_text(json.dumps({"version": 1, "destinations": [
                {"id": "local", "type": "local", "path": root, "enabled": False}
            ]}))
            self.assertEqual(module.load_config(path), [])

    def test_local_archive_verified_and_manifest_last(self):
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            source = root / "20261026T190000Z-42-7.tar.age"
            source.write_bytes(b"test ciphertext")
            source.with_name("20261026T190000Z-42-7.release.env").write_text("IMAGE_ID=sha256:abc\n")
            target = root / "target"
            target.mkdir()
            row = {"type": "local", "path": str(target)}
            module.publish(row, source)
            manifest = json.loads((target / (source.name + ".complete.json")).read_text())
            self.assertEqual(manifest["files"][0]["sha256"], module.digest(source))
            self.assertEqual((target / source.name).read_bytes(), source.read_bytes())
            module.publish(row, source)  # completed retry is idempotent
            source.write_bytes(b"changed")
            with self.assertRaises(module.BackupError):
                module.publish(row, source)
            self.assertEqual((target / source.name).read_bytes(), b"test ciphertext")

    def test_plaintext_refused(self):
        with tempfile.TemporaryDirectory() as root:
            source = Path(root) / "backup.tar"
            source.write_bytes(b"secret")
            with self.assertRaises(module.BackupError):
                module.publish({"type": "local", "path": root}, source)

    def test_nfs_cannot_fall_back_to_local_disk(self):
        with tempfile.TemporaryDirectory() as root, patch.object(module, "run", return_value=b"ext4\n"):
            with self.assertRaises(module.BackupError):
                module.preflight({"type": "nfs", "path": root})

    def test_fetch_checks_manifest_and_never_restores_live_data(self):
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            source = root / "20261026T190000Z-42-7.tar.age"
            source.write_bytes(b"ciphertext")
            source.with_name("20261026T190000Z-42-7.release.env").write_text("IMAGE_ID=sha256:abc")
            remote, recovered = root / "remote", root / "recovered"
            remote.mkdir(); recovered.mkdir()
            row = {"type": "local", "path": str(remote)}
            module.publish(row, source)
            module.fetch(row, source.name, recovered)
            self.assertEqual((recovered / source.name).read_bytes(), source.read_bytes())
            (remote / source.name).write_bytes(b"corrupted")
            with self.assertRaises(module.BackupError):
                module.fetch(row, source.name, recovered)
            self.assertEqual((recovered / source.name).read_bytes(), b"ciphertext")

    def test_types_and_paths(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / "destinations.json"
            for kind in ["s3", "sftp", "webdav", "smb"]:
                row = {"id": kind, "type": kind, "remote": "offsite", "path": "bucket/mailwoman", "rcloneConfig": str(Path(root) / "rclone.conf")}
                path.write_text(json.dumps({"version": 1, "destinations": [row]}))
                self.assertEqual(module.load_config(path)[0]["type"], kind)
                row["path"] = "../escape"
                path.write_text(json.dumps({"version": 1, "destinations": [row]}))
                with self.assertRaises(module.BackupError):
                    module.load_config(path)

    def test_sftp_pin_required_and_webdav_tls(self):
        with tempfile.TemporaryDirectory() as root, patch.object(module.shutil, "which", return_value="rclone"):
            config = Path(root) / "rclone.conf"
            row = {"type": "sftp", "remote": "offsite", "rcloneConfig": str(config)}
            config.write_text("[offsite]\ntype = sftp\nhost = example.test\n")
            with self.assertRaises(module.BackupError):
                module.preflight(row)
            config.write_text("[offsite]\ntype = sftp\nknown_hosts_file = none\n")
            with self.assertRaises(module.BackupError):
                module.preflight(row)
            config.write_text("[offsite]\ntype = sftp\nssh = ssh -o StrictHostKeyChecking=no -o StrictHostKeyChecking=yes -o BatchMode=yes host\n")
            with self.assertRaises(module.BackupError):
                module.preflight(row)
            config.write_text("[offsite]\ntype = webdav\nurl = http://example.test\n")
            row["type"] = "webdav"
            with self.assertRaises(module.BackupError):
                module.preflight(row)

    def test_custom_s3_endpoint_requires_https_without_network_access(self):
        with tempfile.TemporaryDirectory() as root, patch.object(module.shutil, "which", return_value="rclone"), \
             patch.object(module, "run") as run:
            config = Path(root) / "rclone.conf"
            row = {"type": "s3", "remote": "offsite", "rcloneConfig": str(config)}
            config.write_text("[offsite]\ntype = s3\nendpoint = http://storage.example.invalid\n")
            with self.assertRaisesRegex(module.BackupError, "HTTPS"):
                module.preflight(row)
            for endpoint in ["https://storage.example.invalid", ""]:
                config.write_text(f"[offsite]\ntype = s3\nendpoint = {endpoint}\n")
                module.preflight(row)
            run.assert_not_called()

    def test_rclone_overrides_cannot_bypass_validated_transport_or_immutability(self):
        with tempfile.TemporaryDirectory() as root, patch.object(module.shutil, "which", return_value="rclone"):
            config = Path(root) / "rclone.conf"
            base = "[offsite]\ntype = s3\nendpoint = https://storage.example.invalid\n"
            row = {"type": "s3", "remote": "offsite", "rcloneConfig": str(config)}
            for option in ["global.immutable", "override.no_check_certificate"]:
                config.write_text(base + f"{option} = secret-value-do-not-print\n")
                with self.assertRaises(module.BackupError) as error:
                    module.preflight(row)
                self.assertIn(option, str(error.exception))
                self.assertNotIn("secret-value-do-not-print", str(error.exception))
            config.write_text(base)
            for name in ["RCLONE_CONFIG_OFFSITE_ENDPOINT", "RCLONE_NO_CHECK_CERTIFICATE", "RCLONE_SFTP_KNOWN_HOSTS_FILE"]:
                with patch.dict(module.os.environ, {name: "secret-value-do-not-print"}), self.assertRaises(module.BackupError) as error:
                    module.preflight(row)
                self.assertIn(name, str(error.exception))
                self.assertNotIn("secret-value-do-not-print", str(error.exception))
            with patch.dict(module.os.environ, {"RCLONE_CA_CERT": str(Path(root) / "ca.pem"),
                                               "HTTPS_PROXY": "http://proxy.example.invalid:8080"}):
                module.preflight(row)
                self.assertEqual(module.os.environ["HTTPS_PROXY"], "http://proxy.example.invalid:8080")

    def test_external_ssh_checks_must_be_options_not_ignored_command_arguments(self):
        valid = [
            "ssh -o StrictHostKeyChecking=yes -o BatchMode=yes backup-host",
            "ssh -vv -F operator.conf -i key -p 2222 -J jump -oStrictHostKeyChecking=yes -oBatchMode=yes user@backup-host",
        ]
        for command in valid:
            self.assertTrue(module.external_ssh_is_pinned(command))
        for command in [
            "ssh backup-host echo -oStrictHostKeyChecking=yes -oBatchMode=yes",
            "ssh backup-host -oStrictHostKeyChecking=yes -oBatchMode=yes",
            "ssh -oStrictHostKeyChecking=no -oStrictHostKeyChecking=yes -oBatchMode=yes backup-host",
            "ssh -oStrictHostKeyChecking=yes -oBatchMode=no backup-host",
            "ssh -oStrictHostKeyChecking=yes -oBatchMode=yes backup-host command",
            "ssh -oStrictHostKeyChecking=yes -oBatchMode=yes --",
        ]:
            self.assertFalse(module.external_ssh_is_pinned(command), command)

    def test_remote_copy_is_argv_based_and_verified_before_manifest(self):
        with tempfile.TemporaryDirectory() as root:
            source = Path(root) / "backup.tar.age"
            source.write_bytes(b"ciphertext")
            remote_files = {}
            commands = []
            def execute(args, timeout, output=False):
                commands.append(args)
                start, end = args[-2:]
                if start.startswith("offsite:"):
                    Path(end).write_bytes(remote_files[start])
                else:
                    remote_files[end] = Path(start).read_bytes()
            row = {"type": "s3", "path": "bucket/path", "remote": "offsite", "rcloneConfig": str(Path(root) / "secret config.ini")}
            with patch.object(module, "run", side_effect=execute):
                module.copy_file(row, source, source.name)
            self.assertEqual(len(commands), 2)
            self.assertIn("--immutable", commands[0])
            self.assertIn(row["rcloneConfig"], commands[0])
            self.assertEqual(remote_files["offsite:bucket/path/backup.tar.age"], b"ciphertext")


    def nextcloud_row(self, root, **extra):
        secret = Path(root) / "nextcloud-app-password"
        secret.write_text("AbCde-FgHij-KlMno-PqRst-UvWxy" + chr(10))
        secret.chmod(0o600)
        row = {"id": "cloud", "type": "nextcloud", "url": "https://cloud.example/nc/", "user": "backup user",
               "appPasswordFile": str(secret), "path": "Backups/mailwoman"}
        row.update(extra)
        return row

    def test_nextcloud_is_a_validated_webdav_preset(self):
        with tempfile.TemporaryDirectory() as root:
            config = Path(root) / "targets.json"
            row = self.nextcloud_row(root)
            config.write_text(json.dumps({"version": 1, "destinations": [row]}))
            self.assertEqual(module.load_config(config)[0]["type"], "nextcloud")
            self.assertEqual(module.nextcloud_root(row), "https://cloud.example/nc/remote.php/dav/files/backup%20user/")
            for bad in [{"url": "http://cloud.example"}, {"url": "https://u:p@cloud.example"}, {"url": "https://cloud.example/?x=1"},
                        {"url": "https://cloud.example/remote.php/dav/files/x"}, {"user": "a/b"}, {"user": ""},
                        {"remote": "x"}, {"rcloneConfig": "/etc/rclone.conf"}, {"appPasswordFile": "relative"},
                        {"chunkSizeMiB": -1}, {"chunkSizeMiB": "10"}, {"path": "../escape"}, {"keep": 0}, {"keepDays": True}]:
                with self.subTest(bad=bad):
                    config.write_text(json.dumps({"version": 1, "destinations": [dict(row, **bad)]}))
                    with self.assertRaises(module.BackupError):
                        module.load_config(config)
            config.write_text(json.dumps({"version": 1, "destinations": [
                {"id": "dav", "type": "webdav", "remote": "dav", "rcloneConfig": "/etc/rclone.conf", "path": "x", "user": "a"}]}))
            with self.assertRaises(module.BackupError):
                module.load_config(config)

    @unittest.skipIf(module.os.name == "nt", "POSIX permission bits")
    def test_nextcloud_app_password_file_must_be_private(self):
        with tempfile.TemporaryDirectory() as root:
            row = self.nextcloud_row(root)
            self.assertEqual(module.read_app_password(row), "AbCde-FgHij-KlMno-PqRst-UvWxy")
            Path(row["appPasswordFile"]).chmod(0o644)
            with self.assertRaises(module.BackupError):
                module.read_app_password(row)

    def test_nextcloud_folder_maps_each_failure_to_a_clear_error(self):
        with tempfile.TemporaryDirectory() as root:
            row = self.nextcloud_row(root)
            calls = []
            def server(statuses):
                def dav(method, url, user, password, depth=None, timeout=30):
                    calls.append((method, url))
                    return statuses.pop(0)
                return dav
            with patch.object(module, "dav", side_effect=server([(207, b""), (404, b""), (201, b""), (404, b""), (201, b"")])):
                self.assertTrue(module.nextcloud_folder(row, "pw"))
            self.assertEqual([c[0] for c in calls], ["PROPFIND", "PROPFIND", "MKCOL", "PROPFIND", "MKCOL"])
            self.assertTrue(calls[-1][1].endswith("/remote.php/dav/files/backup%20user/Backups/mailwoman/"))
            for statuses, words in [([(401, b"")], "app password"), ([(404, b"")], "username"), ([(302, b"")], "redirected"),
                                    ([(207, b""), (404, b""), (403, b"")], "could not create"),
                                    ([(207, b""), (207, b"<d:resourcetype/>")], "is a file")]:
                with self.subTest(words=words), patch.object(module, "dav", side_effect=server(statuses)):
                    with self.assertRaises(module.BackupError) as raised:
                        module.nextcloud_folder(row, "pw")
                    self.assertIn(words, str(raised.exception))
                    self.assertNotIn("pw", str(raised.exception).replace("password", ""))

    def test_retention_keeps_newest_complete_sets_and_never_the_current_one(self):
        with tempfile.TemporaryDirectory() as root:
            folder = Path(root)
            names = [f"2026{month:02d}01T000000Z-1-1.tar.age" for month in range(1, 7)]
            for name in names:
                (folder / name).write_bytes(b"x")
                (folder / (name + ".complete.json")).write_text("{}")
                (folder / name.replace(".tar.age", ".release.env")).write_text("IMAGE_ID=x" + chr(10))
            incomplete = "20250101T000000Z-1-1.tar.age"
            (folder / incomplete).write_bytes(b"partial")
            (folder / "unrelated.txt").write_text("keep me")
            row = {"type": "local", "path": str(folder), "keep": 2}
            removed = module.retention(row, names[0])
            self.assertEqual(sorted(removed), sorted(names[1:4]))
            for name in [names[0], names[4], names[5], incomplete, "unrelated.txt"]:
                self.assertTrue((folder / name).exists(), name)
            for name in names[1:4]:
                self.assertFalse((folder / name).exists())
                self.assertFalse((folder / (name + ".complete.json")).exists())
                self.assertFalse((folder / name.replace(".tar.age", ".release.env")).exists())
            now = module.datetime.datetime(2026, 7, 1, tzinfo=module.datetime.timezone.utc)
            self.assertEqual(module.retention({"type": "local", "path": str(folder), "keep": 1, "keepDays": 70}, names[0], now), [])
            self.assertEqual(module.retention({"type": "local", "path": str(folder), "keep": 1, "keepDays": 20}, names[0], now), [names[4]])
            self.assertEqual(module.retention({"type": "local", "path": str(folder)}, ""), [])

    def test_test_command_writes_and_removes_a_probe(self):
        with tempfile.TemporaryDirectory() as root:
            target = Path(root) / "target"
            target.mkdir()
            config = Path(root) / "targets.json"
            config.write_text(json.dumps({"version": 1, "destinations": [{"id": "local", "type": "local", "path": str(target)}]}))
            with patch.object(sys, "argv", ["backup-destinations.py", "test", "--config", str(config)]),                  contextlib.redirect_stdout(io.StringIO()) as stdout:
                self.assertEqual(module.main(), 0)
            self.assertIn("local: test complete", stdout.getvalue())
            self.assertEqual(list(target.iterdir()), [])

if __name__ == "__main__":
    unittest.main()
