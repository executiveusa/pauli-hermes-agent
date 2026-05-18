import json

from tools.skills_tool import skill_view


def test_skill_view_supports_absolute_skill_directory(tmp_path):
    skill_dir = tmp_path / "custom-skill"
    skill_dir.mkdir()
    (skill_dir / "SKILL.md").write_text(
        "---\nname: custom-skill\ndescription: Absolute path skill\n---\n\n# Custom\n"
    )
    payload = json.loads(skill_view(str(skill_dir), preprocess=False))
    assert payload["success"] is True
    assert payload["name"] == "custom-skill"
    assert payload["skill_dir"] == str(skill_dir)
