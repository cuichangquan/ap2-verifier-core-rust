# One-time synthetic conformance export. Private keys remain in process memory.
require 'json'
require '/baseline/lib/ap2_mandate_verifier'
require '/baseline/test/support/multi_item_fixture'
b=AP2MandateVerifier
path='/work/conformance/v1/cases.json'
cases=JSON.parse(File.read(path));raise 'already extracted' if cases.any?{|c|c['id'].start_with?('causal-')}
variants={
 'shortage'=>{checkout:MultiItemFixture.checkout([MultiItemFixture.line('a','A',1),MultiItemFixture.line('b','B',1,250)])},
 'extra'=>{checkout:MultiItemFixture.checkout([MultiItemFixture.line('a','A',3),MultiItemFixture.line('b','B',1,250)])},
 'wrong-product'=>{checkout:MultiItemFixture.checkout([MultiItemFixture.line('a','C',2),MultiItemFixture.line('b','B',1,250)])},
 'empty-candidates'=>{requirements:[MultiItemFixture.requirement('r',1,[])]},
 'unknown-constraint'=>{open_transform:->(o){o['constraints'][0]['type']='unknown'}},
 'fractional-requirement'=>{open_transform:->(o){o['constraints'][0]['items'][0]['quantity']=1.0}},
 'duplicate-requirement-id'=>{open_transform:->(o){o['constraints'][0]['items'][1]['id']=o['constraints'][0]['items'][0]['id']}},
 'extra-open-field'=>{open_transform:->(o){o['extra']=true}},
 'wrong-agent'=>{open_transform:->(o){o['cnf']['jwk']['x']='A'*43}},
 'unknown-graph'=>{open_transform:->(o){o['_sd']=['A'*43]}}
}
variants.each do |id,options|
 f=MultiItemFixture.build(**options)
 inputs=b::InputBuilder.inputs(**f[:args]);r=b.verify_checkout(**inputs)
 raise 'expected reject' unless r.rejected?
 cases << {id:"causal-#{id}",operation:'verify_checkout',arguments:inputs,expected_status:'rejected',expected_code:r.code.to_s}
end
File.write(path,JSON.pretty_generate(cases)+"\n")
puts JSON.generate(total_cases:cases.size,added:variants.size)
