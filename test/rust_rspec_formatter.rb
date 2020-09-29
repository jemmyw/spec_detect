require "json"
require "rspec/core"

class RustRspecFormatter
  RSpec::Core::Formatters.register self, :start, :stop, :example_started, :example_passed, :example_failed

  def initialize(output)
    @output = output
  end

  def dump_notification(type, notification = {})
    @output.puts({ type: type }.merge(notification.to_h).to_json)
    @output.flush
  end

  def start(notification)
    dump_notification("start", notification)
  end

  def stop(notification)
    dump_notification("stop")
  end

  def example_started(notification)
    dump_notification("example_started", {
      id: notification.example.id,
      location: notification.example.location,
      description: notification.example.full_description,
    })
  end

  def example_passed(notification)
    dump_notification("example_passed", {
      id: notification.example.id,
      location: notification.example.location,
      description: notification.example.full_description,
      run_time: notification.example.execution_result.run_time
    })
  end

  def example_failed(notification)
    dump_notification("example_failed", {
      id: notification.example.id,
      location: notification.example.location,
      description: notification.example.full_description,
      run_time: notification.example.execution_result.run_time,
      exception: notification.example.execution_result.exception.to_s,
    })
  end
end