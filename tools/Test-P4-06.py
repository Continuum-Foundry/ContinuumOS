"""Host oracle negatives are separate from real device/guest evidence."""
import copy,importlib.util,unittest
spec=importlib.util.spec_from_file_location('oracle','tools/Review-P4-06.py');r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
class Oracle(unittest.TestCase):
    def test_reviewer_repair_rejects_any_guest_change_or_incomplete_allowlist(self):
        import p4_input_acceptance as gate
        a={p:'old'for p in gate.REPAIR_FILES};a['kernel/src/input_guest.rs']='same';b={**a,**{p:'new'for p in gate.REPAIR_FILES}}
        gate.repair_changes(a,b)
        with self.assertRaises(ValueError):gate.repair_changes(a,{**b,'kernel/src/input_guest.rs':'changed'})
        with self.assertRaises(ValueError):gate.repair_changes(a,a)
    def fixture(self):
        return [dict(device='0',kind=str(k),code=str(c),value=str(v)) for c,v in [(42,1),(54,1),(42,0),(30,1),(30,1),(30,0),(54,0)]+[(30,1)]*32+[(30,0),(42,1)] for k,c,v in [(1,c,v),(0,0,0)]]
    def test_fixture_requires_distinct_bytes_but_ordinary_links_may_match(self):
        r.clients([dict(sha256='a'),dict(sha256='a')],False)
        with self.assertRaises(ValueError):r.clients([dict(sha256='a'),dict(sha256='a')],True)
        r.clients([dict(sha256='a'),dict(sha256='b')],True)
    def test_literal_reports_and_missing_release(self):
        data=self.fixture();r.keyboard(data);self.assertEqual(len(data),82);self.assertEqual(data[-2],dict(device='0',kind='1',code='42',value='1'))
    def test_missing_surplus_reordered_repeat_and_wrong_value_reject(self):
        original=self.fixture()
        candidates=[original[:-1],original+[original[-1]],original[2:]+original[:2]]
        for key,value in [('device','1'),('kind','9'),('code','999'),('value','2')]:
            data=copy.deepcopy(original);data[8][key]=value;candidates.append(data)
        for data in candidates:
            with self.assertRaises(ValueError):r.keyboard(data)
if __name__=='__main__':unittest.main()
