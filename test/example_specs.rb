require "rspec"

RSpec.describe do
  before { sleep 0.1 }

  1.times do
    describe "passing specs", passing: true do
      it "passes 1" do
        expect(true).to eq true
      end

      it "is slow" do
        sleep 2
        expect(true).to eq true
      end

      it "passes 2" do
        expect(false).to eq false
      end

      it "passes longly" do
        expect(true).to eq true
      end
    end

    describe "failing specs", failing: true do
      it "fails 1" do
        expect(true).to eq false
      end

      it "fails 2" do
        expect(false).to eq true
      end
    end
  end
end
