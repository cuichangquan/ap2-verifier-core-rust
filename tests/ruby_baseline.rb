require '/baseline/lib/ap2_mandate_verifier'
require_relative '../adapters/ruby/lib/ap2_verifier_core'
require 'digest'
manifest=JSON.parse(File.read('/work/conformance/v1/manifest.json'))
manifest['files'].each{|path,record|raise 'fixture digest mismatch' unless Digest::SHA256.file('/work/'+path).hexdigest==record['sha256']}
core=AP2VerifierCore::Native.new(library_path:'/work/target/release/libap2_verifier_core.so')
count=0
JSON.parse(File.read('/work/conformance/v1/cases.json')).each do |c|
 next unless c['operation']=='verify_checkout'
 result=AP2MandateVerifier.verify_checkout(**c['arguments'].transform_keys(&:to_sym))
 native=core.verify(operation:c['operation'],arguments:c['arguments'])
 raise "status drift #{c['id']}" unless native['status']==result.status.to_s
 if result.success?
  ruby=result.evidence.to_h.reject{|key,_|%w[contract_version implementation_revision].include?(key)}
  rust=native['evidence'].reject{|key,_|%w[contract_version implementation_revision].include?(key)}
  raise "evidence drift #{c['id']}" unless ruby==rust
 else
  raise 'failure Evidence' unless native['evidence'].nil? && !native['verified']
 end
 count+=1
end
puts JSON.generate(complete_pipeline_baseline_cases:count,evidence_projection_match:true,fixture_hashes_verified:true)
