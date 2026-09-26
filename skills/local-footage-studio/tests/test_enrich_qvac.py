from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "scripts" / "enrich_qvac.py"
SPEC = importlib.util.spec_from_file_location("enrich_qvac", MODULE_PATH)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class FrameSelectionTests(unittest.TestCase):
    def test_downsampling_preserves_order_and_matching_timestamps(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            frame_dir = Path(directory)
            for index, timestamp in enumerate((0.0, 5.0, 10.0, 15.0, 20.0)):
                (frame_dir / f"frame-{index:03d}-t{timestamp:.3f}.jpg").write_bytes(b"jpeg")

            selected = MODULE.select_frames(frame_dir, 3)

            self.assertEqual([timestamp for _, timestamp in selected], [0.0, 10.0, 20.0])
            self.assertEqual([path.name for path, _ in selected], [
                "frame-000-t0.000.jpg",
                "frame-002-t10.000.jpg",
                "frame-004-t20.000.jpg",
            ])

    def test_rejects_frame_without_timestamp_marker(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            frame_dir = Path(directory)
            (frame_dir / "frame.jpg").write_bytes(b"jpeg")
            with self.assertRaisesRegex(RuntimeError, "lacks timestamp"):
                MODULE.select_frames(frame_dir, 2)


class ModelJsonTests(unittest.TestCase):
    def valid_payload(self) -> dict[str, object]:
        return {
            "observed": ["Fixed camera facing trees"],
            "temporal_change": ["Light decreases across frames"],
            "inference": ["Possible sunset"],
            "clip_type": "timelapse",
            "confidence": 0.81,
            "search_terms": ["trees", "sunset"],
        }

    def test_accepts_plain_json(self) -> None:
        result = MODULE.parse_model_json(json.dumps(self.valid_payload()))
        self.assertEqual(result["clip_type"], "timelapse")
        self.assertEqual(result["confidence"], 0.81)

    def test_accepts_fenced_json(self) -> None:
        content = "```json\n" + json.dumps(self.valid_payload()) + "\n```"
        result = MODULE.parse_model_json(content)
        self.assertEqual(result["search_terms"], ["trees", "sunset"])

    def test_rejects_unknown_clip_type(self) -> None:
        payload = self.valid_payload()
        payload["clip_type"] = "cinematic-masterpiece"
        with self.assertRaisesRegex(RuntimeError, "Unsupported clip_type"):
            MODULE.parse_model_json(json.dumps(payload))

    def test_rejects_empty_evidence_strings(self) -> None:
        payload = self.valid_payload()
        payload["observed"] = [""]
        with self.assertRaisesRegex(RuntimeError, "non-empty strings"):
            MODULE.parse_model_json(json.dumps(payload))

    def test_rejects_out_of_range_confidence(self) -> None:
        payload = self.valid_payload()
        payload["confidence"] = 1.2
        with self.assertRaisesRegex(RuntimeError, "between 0 and 1"):
            MODULE.parse_model_json(json.dumps(payload))


if __name__ == "__main__":
    unittest.main()
