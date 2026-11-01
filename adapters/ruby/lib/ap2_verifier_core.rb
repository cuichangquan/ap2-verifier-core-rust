# frozen_string_literal: true
require "fiddle"
require "json"
module AP2VerifierCore
  # Explicit absolute local library path, never token-selected or downloaded.
  class Native
    def initialize(library_path:)
      raise ArgumentError, "absolute library path required" unless library_path.instance_of?(String) && library_path.start_with?("/")
      @handle = Fiddle.dlopen(library_path)
      @verify = Fiddle::Function.new(@handle["ap2_verify_v1"],
        [Fiddle::TYPE_VOIDP, Fiddle::TYPE_SIZE_T, Fiddle::TYPE_VOIDP, Fiddle::TYPE_SIZE_T], Fiddle::TYPE_SSIZE_T)
    end
    def verify(operation:, arguments:)
      request = JSON.generate(contract_version: "ap2-core-request.1", operation: operation, arguments: arguments)
      verify_json(request)
    end
    def verify_json(request)
      raise ArgumentError, "String required" unless request.instance_of?(String)
      if request.bytesize > 131_072
        return freeze_tree({"contract_version" => "ap2-core-result.1", "status" => "rejected", "code" => "input_too_large", "verified" => false, "facts" => nil, "evidence" => nil})
      end
      input = request.b.dup.freeze
      output = Fiddle::Pointer.malloc(262_144, Fiddle::RUBY_FREE)
      size = @verify.call(input, input.bytesize, output, 262_144)
      raise "native verification unavailable" unless size.between?(1, 262_144)
      result = JSON.parse(output[0, size])
      raise "native result contract mismatch" unless result["contract_version"] == "ap2-core-result.1"
      freeze_tree(result)
    end
    private
    def freeze_tree(value)
      case value
      when Hash then value.each { |key, child| key.freeze; freeze_tree(child) }
      when Array then value.each { |child| freeze_tree(child) }
      end
      value.freeze
    end
  end
end
