# Synthetic local verification only; fixed clock and original signed bytes.
require_relative '../adapters/ruby/lib/ap2_verifier_core'
cases=JSON.parse(File.read(File.expand_path('../conformance/v1/cases.json',__dir__)))
inputs=cases.find{|c|c['id']=='single-host'}.fetch('arguments')
path=ENV.fetch('AP2_CORE_LIBRARY',File.expand_path('../target/release/libap2_verifier_core.so',__dir__))
core=AP2VerifierCore::Native.new(library_path:path)
result=core.verify(operation:'verify_checkout',arguments:inputs)
puts "#{result['status']}/#{result['code']}; evidence=#{!result['evidence'].nil?}"
raise 'Verification failed; stop' unless result['verified']
# No operation is authorized/consumed/executed by this example.
