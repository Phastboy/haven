"""Credential-free provisioning regression tests."""
import json
import unittest
from cloudinary_transforms import MANIFEST, ProvisionError, reconcile


class FakeAPI:
    def __init__(self, states=None):
        self.states = states or {}
        self.writes = []

    def request(self, method, name, fields=None):
        if method == 'GET':
            return self.states.get(name)
        self.writes.append((method, name, fields))
        if method == 'POST':
            self.states[name] = {'info': fields['transformation'], 'allowed_for_strict': True}
        elif method == 'PUT':
            self.states[name]['allowed_for_strict'] = True
        return {}


class ProvisionTests(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads(MANIFEST.read_text())

    def test_missing_check_is_read_only(self):
        api = FakeAPI()
        with self.assertRaises(ProvisionError):
            reconcile(api, self.manifest)
        self.assertEqual(api.writes, [])

    def test_create_and_repeat(self):
        api = FakeAPI()
        reconcile(api, self.manifest, True)
        self.assertEqual(len(api.writes), 10)
        api.writes.clear()
        reconcile(api, self.manifest, True)
        reconcile(api, self.manifest)
        self.assertEqual(api.writes, [])

    def test_conflict_preflights_before_writes(self):
        api = FakeAPI({'detail_2x': {'info': 'c_fill,w_1,h_1,f_webp'}})
        with self.assertRaisesRegex(ProvisionError, 'conflicting'):
            reconcile(api, self.manifest, True)
        self.assertEqual(api.writes, [])

    def test_strict_allowance_repaired_without_definition_update(self):
        api = FakeAPI()
        reconcile(api, self.manifest, True)
        api.states['card_sq']['allowed_for_strict'] = False
        api.writes.clear()
        with self.assertRaises(ProvisionError):
            reconcile(api, self.manifest)
        reconcile(api, self.manifest, True)
        self.assertEqual(api.writes, [('PUT', 'card_sq', {'allowed_for_strict': 'true'})])

    def test_parameter_order_does_not_conflict(self):
        api = FakeAPI()
        reconcile(api, self.manifest, True)
        api.states['card_sq']['info'] = 'f_webp,h_320,w_320,c_fill'
        reconcile(api, self.manifest)

    def test_documented_api_info_array(self):
        api = FakeAPI()
        reconcile(api, self.manifest, True)
        api.states['card_sq']['info'] = [{'crop': 'fill', 'width': 320, 'height': 320, 'format': 'webp'}]
        reconcile(api, self.manifest)

    def test_postcheck_failure_is_error(self):
        class BrokenAPI(FakeAPI):
            def request(self, method, name, fields=None):
                if method == 'GET':
                    return None
                return super().request(method, name, fields)
        with self.assertRaisesRegex(ProvisionError, 'verification failed'):
            reconcile(BrokenAPI(), self.manifest, True)


if __name__ == '__main__':
    unittest.main()
