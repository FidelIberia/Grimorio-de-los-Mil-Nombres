from grimorio_ru_names import SCHEMA_ID, build_catalog, catalog_dict, decline, emit_json
from grimorio_ru_names.names import OVERRIDE
from grimorio_ru_names.validate import validate_catalog


def test_catalog_non_empty_and_unique():
    cat = build_catalog()
    assert len(cat) >= 100
    names = [n.name for n in cat]
    assert len(names) == len(set(names))


def test_zeus_override():
    patron, verbal = decline("Зевс", OVERRIDE)
    assert patron == "Зевса"
    assert verbal == "Зевсом"


def test_contract_envelope_validates():
    data = catalog_dict()
    assert data["schema"] == SCHEMA_ID
    assert validate_catalog(data) == []


def test_json_roundtrip_shape():
    import json

    raw = emit_json()
    data = json.loads(raw)
    assert validate_catalog(data) == []
    assert data["names"][0]["name"] == "Зевс"
