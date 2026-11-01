require_relative '../adapters/ruby/lib/ap2_verifier_core'
require 'digest'
core=AP2VerifierCore::Native.new(library_path:ENV.fetch('AP2_CORE_LIBRARY','/work/target/debug/libap2_verifier_core.so'))
cases=JSON.parse(File.read('/work/conformance/v1/cases.json'))
count=0
cases.each do |c|
 result=core.verify(operation:c['operation'],arguments:c['arguments'])
 raise "#{c['id']}: #{result['code']}" unless result['status']==c['expected_status']
 raise 'mutable native results' unless result.frozen? && (result['evidence'].nil? || result['evidence'].frozen?)
 count+=1
end
%w[single-host multi-host].each do |id|
 c=cases.find{|v|v['id']==id}
 base=c['arguments'];host=JSON.parse(base['host_json'])
 mutations=[]
 [ ['challenge','state','consumed'],['challenge','nonce','other'],['binding','order_version','other'],['binding','expected_total',999],['challenge','not_after',1800000000],['challenge','trust_content_hash','0'*64] ].each do |outer,key,value|
   h=Marshal.load(Marshal.dump(host));h[outer][key]=value;mutations << base.merge('host_json'=>JSON.generate(h))
 end
 %w[root_presentation terminal_jws host_checkout_jws trust_json selection_json].each{|key|mutations << base.merge(key=>base[key]+'x')}
 mutations << base.merge('verified_at'=>1800001000)
 mutations.each do |a|
  r=core.verify(operation:'verify_checkout',arguments:a)
  raise 'fail-open host mutation' if r['verified'] || !r['evidence'].nil?
  count+=1
 end
end
# Same instance used by parallel callers; independent caller-owned output memory.
threads=8.times.map do
 Thread.new{25.times{c=cases.find{|v|v['id']=='direct'};r=core.verify(operation:'direct',arguments:c['arguments']);raise 'concurrency' unless r['verified']}}
end
threads.each(&:value)
# Host accepts only exact trusted library path. No Ruby verifier/subprocess is loaded.
raise 'source Gem unexpectedly loaded' if defined?(AP2MandateVerifier)
puts JSON.generate(ruby_ffi_cases:count,concurrent_calls:200,source_gem_loaded:false)
# A String's singleton overrides must not supply a false pointer/length pair.
wire='{}'.dup
%i[instance_of? bytesize b dup].each{|name|wire.define_singleton_method(name){|*|raise 'caller callback dispatched'}}
raise 'snapshot callback' unless core.verify_json(wire)['status']=='rejected'
large=' '*131_073
large.define_singleton_method(:bytesize){0}
raise 'size override' unless core.verify_json(large)['code']=='input_too_large'
pretender=Object.new
pretender.define_singleton_method(:instance_of?){|*|true}
begin
 core.verify_json(pretender)
 raise 'type override'
rescue ArgumentError
end
puts JSON.generate(string_snapshot_override_checks:3)
