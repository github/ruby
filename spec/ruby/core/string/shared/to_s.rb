describe :string_to_s, shared: true do
  it "returns self when self.class == String" do
    a = "a string"
    a.should equal(a.send(@method))
  end

  it "returns a new instance of String when called on a subclass" do
    a = StringSpecs::MyString.new("a string")
    s = a.send(@method)
    s.should == "a string"
    s.should be_an_instance_of(String)
  end

  ruby_version_is ''...'2.7' do
    it "taints the result when self is tainted" do
      "x".taint.send(@method).should.tainted?
      StringSpecs::MyString.new("x").taint.send(@method).should.tainted?
    end
  end
end
