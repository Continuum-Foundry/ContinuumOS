"""Independent pixel and orchestration policy tests for P4-05."""
import collections, json, unittest
from pathlib import Path
import p4_surface_acceptance as p
import story_validation as v
class Tests(unittest.TestCase):
    def test_counted_schema2_plan(self):
        m=v.validate_manifest(json.loads(Path('config/p4-05-validation.json').read_text()));plan=v.execution_plan(m,p.checks(m))
        self.assertEqual(plan['vm_launches'],22);self.assertEqual(plan['separate_metadata_builds'],20);self.assertFalse(plan['full_suites']);self.assertIn('retention',plan)
        m['execution_budget']['max_vm_launches']=21
        with self.assertRaises(ValueError):v.execution_plan(m,p.checks(m))
    def test_reference_opaque_translucent_clip_and_exposure(self):
        frames={n:p.oracle.pixels(1280,720,n) for n in range(1,7)}
        pixel=lambda n,x,y:frames[n][(y*1280+x)*3:(y*1280+x+1)*3]
        self.assertEqual(pixel(1,0,25),bytes([180,40,20]));self.assertEqual(pixel(1,149,144),bytes([110,110,70]));self.assertEqual(pixel(1,150,144),bytes([26,99,72]))
        self.assertEqual(pixel(1,87,74),bytes([180,40,20]));self.assertEqual(pixel(2,88,74),bytes([180,40,20]));self.assertEqual(pixel(2,248,164),bytes([26,99,72]))
        self.assertEqual(frames[3],frames[4]);self.assertEqual(frames[3],frames[6]);self.assertEqual(frames[1],frames[5]);self.assertNotEqual(frames[1],frames[2])
    def test_partial_start_unwind_is_distinct_from_compositor_panic(self):
        records=[{'event':'m1.desktop_reap','fields':{'role':'compositor','result':'513'}},
                 {'event':'m1.desktop_reap','fields':{'role':'compositor','result':'90'}},
                 {'event':'m1.desktop_launch','fields':{'role':'editor','mode':'2','authority_session':'2002'}}]
        result=p.oracle.runtime_baseline(records,7)
        self.assertEqual(result[0]['fields']['result'],'513')
        self.assertEqual(result[1]['fields']['result'],'0')
        self.assertEqual(records[1]['fields']['result'],'90')
        records[1]['fields']['result']='0'
        with self.assertRaises(ValueError):p.oracle.runtime_baseline(records,7)
    def test_missing_raw_criteria_reject(self):
        for scenario in range(1,7):
            with self.assertRaises((ValueError,KeyError,StopIteration)):p.oracle.check([],1280,720,scenario)
if __name__=='__main__':unittest.main()
