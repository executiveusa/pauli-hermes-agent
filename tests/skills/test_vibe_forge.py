import importlib.util
import unittest
from pathlib import Path
p=Path(__file__).resolve().parents[2]/"skills"/"vibe-forge"/"scripts"/"factory.py"
spec=importlib.util.spec_from_file_location("factory",p)
factory=importlib.util.module_from_spec(spec);spec.loader.exec_module(factory)
class FactoryTests(unittest.TestCase):
    def test_initial_authority_hold(self):
        r=factory.route({"product":"New product"});self.assertEqual((r["stage"],r["status"]),("00","HOLD"))
    def test_pbn_research_required(self):
        r=factory.route({"product":"New product","passed_stages":factory.NAMES[:2]})
        self.assertIn("UNRESEARCHED",r["blocked_reason"])
    def test_dean_route(self):
        r=factory.route({"product":"New product","passed_stages":factory.NAMES[:3]})
        self.assertEqual(r["skills"],["deanpeters:pm-skills-navigator"])
    def test_yc_specialization(self):
        r=factory.route({"product":"AI B2B","passed_stages":factory.NAMES[:5],"domain":"b2b-ai"})
        self.assertEqual(r["skills"],["jona:b2b-ai-startup-levie"])
    def test_commercial_evidence_hold(self):
        r=factory.route({"product":"Offer","passed_stages":factory.NAMES[:6]})
        self.assertEqual(r["status"],"HOLD")
    def test_contact_requires_authority(self):
        r=factory.route({"product":"Offer","passed_stages":factory.NAMES[:7]})
        self.assertIn("not authorized",r["blocked_reason"])
    def test_independent_review(self):
        r=factory.route({"product":"Offer","passed_stages":factory.NAMES[:10]})
        self.assertIn("independent",r["blocked_reason"])
    def test_launch_requires_human(self):
        r=factory.route({"product":"Offer","passed_stages":factory.NAMES[:11]})
        self.assertIn("release",r["blocked_reason"])
    def test_no_skip(self):
        with self.assertRaises(ValueError):
            factory.route({"product":"Offer","passed_stages":["commercial"]})
    def test_no_duplicates(self):
        with self.assertRaises(ValueError):
            factory.route({"product":"Offer","passed_stages":["intake","intake"]})
    def test_complete(self):
        r=factory.route({"product":"Offer","passed_stages":factory.NAMES})
        self.assertEqual(r["status"],"COMPLETE_FOR_THIS_CYCLE")
if __name__=="__main__": unittest.main()
