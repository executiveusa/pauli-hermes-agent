import pytest

from tools.llm_council_tool import _borda, _parse_ranking
from tools.terabithia_schedule_tool import _canonical_recurrence


def test_parse_ranking_extracts_final_order_once():
    text = "A is clear. B is weaker.\nFINAL RANKING: C > A > B"
    assert _parse_ranking(text, ["A", "B", "C"]) == ["C", "A", "B"]


def test_parse_ranking_ignores_unknown_and_duplicates():
    assert _parse_ranking("FINAL RANKING: A > Z > A > B", ["A", "B"]) == ["A", "B"]


def test_borda_is_deterministic():
    # A>B>C, B>A>C, A>C>B => A wins, C loses.
    ranked = _borda([[0, 1, 2], [1, 0, 2], [0, 2, 1]], 3)
    assert [idx for idx, _ in ranked] == [0, 1, 2]
    assert ranked[0][1] > ranked[-1][1]


def test_duration_maps_to_fixed_interval():
    assert _canonical_recurrence("30m") == {"kind": "interval", "every_seconds": 1800}


def test_every_duration_maps_to_fixed_interval():
    assert _canonical_recurrence("every 2h") == {"kind": "interval", "every_seconds": 7200}


def test_daily_cron_maps_without_semantic_drift():
    result = _canonical_recurrence("0 9 * * *")
    assert result["kind"] == "interval"
    assert result["every_seconds"] == 86400
    assert "start_at" in result


def test_weekly_cron_maps_without_semantic_drift():
    result = _canonical_recurrence("30 8 * * 1")
    assert result["kind"] == "interval"
    assert result["every_seconds"] == 604800
    assert "start_at" in result


def test_complex_calendar_cron_fails_closed():
    with pytest.raises(ValueError, match="cannot yet be represented losslessly"):
        _canonical_recurrence("0 9 1 * *")
