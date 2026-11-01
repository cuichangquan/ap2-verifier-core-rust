# Synthetic, MIT source extraction only. Baseline pin checked by runner.
require 'json'
require 'digest'
require '/baseline/lib/ap2_mandate_verifier'
require '/baseline/test/support/multi_item_fixture'
abort "frozen suite exists; choose a new version for regeneration" if File.exist?("/work/conformance/v1/manifest.json")
b=AP2MandateVerifier
out='/work/conformance/v1'
File.write("#{out}/single_profile.json",b::TrustContext::PROFILE_BODY)
File.write("#{out}/multi_profile.json",b::TrustContext::MULTI_PROFILE_BODY)
cases=[]
JSON.parse(File.read('/baseline/research/stage15/results/R3/INPUTS.json')).each do |name,a|
 r=b.verify_checkout(**a.transform_keys(&:to_sym)); cases << {id:"legacy-#{name}",operation:'verify_checkout',arguments:a,expected_status:r.status.to_s,expected_code:r.code.to_s}
end
f=MultiItemFixture.build
args=f[:args];inputs=b::InputBuilder.inputs(**args)
scoped=inputs.slice(:root_presentation,:terminal_jws,:host_checkout_jws).merge(trusted_issuer_public_key_pem:f[:keys][0].public_to_pem,expected_agent_public_key_pem:f[:keys][1].public_to_pem,trusted_merchant_public_key_pem:f[:keys][2].public_to_pem,expected_aud:'host',expected_nonce:'test-nonce',verified_at:1800000000,expected_checkout_id:'order',expected_merchant_id:'shop',expected_currency:'USD',expected_total_minor:450,profile_ref:MultiItemFixture::PROFILE)
cases << {id:'multi-flow',operation:'scoped_constraints',arguments:scoped,expected_status:'unverified',expected_code:'scoped_constraints_verified_only'}
cases << {id:'multi-host',operation:'verify_checkout',arguments:inputs,expected_status:'verified_research',expected_code:'scoped_research_verified'}
read=->(n){JSON.parse(File.read("/baseline/examples/input_builder/#{n}.json"))}
trust=b::InputBuilder.trust(snapshot:read.call('trust_snapshot'),tenant_id:'synthetic-tenant',issuer:{'identity'=>'issuer','purpose'=>'checkout','key_ref'=>'issuer-1'},agent:{'identity'=>'agent','purpose'=>'checkout','key_ref'=>'agent-1'},merchant:{'identity'=>'shop','purpose'=>'checkout','key_ref'=>'merchant-1'},verified_at:1800000000)
wires=read.call('signed_wires').transform_keys(&:to_sym)
prepared=b::InputBuilder.prepare(trust:trust,order:read.call('order'),root_presentation:wires[:root_presentation],host_checkout_jws:wires[:host_checkout_jws],challenge_id:'synthetic-challenge',nonce:'n',issuer_id:'synthetic-host',not_before:1799998000,not_after:1800002000)
single=b::InputBuilder.inputs(trust:trust,saved_challenge:prepared,order:read.call('order'),**wires,expected_challenge_issuer_id:'synthetic-host',clock_source_id:'fixed-synthetic-clock',verified_at:1800000000)
cases << {id:'single-host',operation:'verify_checkout',arguments:single,expected_status:'verified_research',expected_code:'scoped_research_verified'}
checkout=args[:host_checkout_jws];merchant=f[:keys][2];provider=f[:keys][0]
direct=b::Buyer.issue_closed(checkout_jwt:checkout,approved_checkout_jwt:checkout,trusted_merchant_public_key_pem:merchant.public_to_pem,expected_merchant_id:'shop',issued_at:1800000000,expires_at:1800000060,signer:b::CheckoutJWT.es256_signer(private_key:provider),public_key_pem:provider.public_to_pem).closed_sd_jwt
cases << {id:'direct',operation:'direct',arguments:{closed_sd_jwt:direct,trusted_provider_public_key_pem:provider.public_to_pem,trusted_merchant_public_key_pem:merchant.public_to_pem,expected_checkout_jwt:checkout,expected_merchant_id:'shop',verified_at:1800000000},expected_status:'verified',expected_code:'direct_checkout_verified'}
[[direct,b::Buyer::PROFILE_REF],[args[:terminal_jws]+'~',MultiItemFixture::PROFILE]].each_with_index do |(wire,profile),i|
 %w[Success Error].each do |outcome|
  opts=outcome=='Success' ? {order_id:'order'} : {error:'declined',error_description:'Synthetic application decision'}
  r=b::CheckoutReceipt.sign(closed_sd_jwt:wire,profile_ref:profile,issuer:'shop',outcome:outcome,issued_at:1800000001,signer:b::CheckoutJWT.es256_signer(private_key:merchant),public_key_pem:merchant.public_to_pem,**opts)
  cases << {id:"receipt-#{i}-#{outcome}",operation:'receipt',arguments:{jwt:r.jwt,closed_sd_jwt:wire,profile_ref:profile,trusted_public_key_pem:merchant.public_to_pem,expected_issuer:'shop',not_before:1800000000,not_after:1800000100,verified_at:1800000002},expected_status:'verified',expected_code:'receipt_verified'}
 end
end
cases << {id:'checkout',operation:'checkout',arguments:{jwt:checkout,trusted_public_key_pem:merchant.public_to_pem,expected_merchant_id:'shop',profile_ref:MultiItemFixture::PROFILE},expected_status:'verified',expected_code:'checkout_verified'}
File.write("#{out}/cases.json",JSON.pretty_generate(cases)+"\n")
puts JSON.generate(cases:cases.size,sha256:Digest::SHA256.file("#{out}/cases.json").hexdigest)
