import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("destinations", Path(__file__).with_name("backup-destinations.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class Destinations(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
