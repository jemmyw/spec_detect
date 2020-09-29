require "socket"

module Forker
  class Client
    def self.new
      p,c = UNIXSocket.pair

      if pid = Kernel.fork
        r = Ruby.new(p, pid)
        r.receive_server
        r
      else
        s = Forker::Server.new(c)
        s.run
        exit
      end
    end

    def self.send_pipes(u, i, o, c)
      u.send_io i
      u.send_io o
      u.send_io u
      u.recv(1)
    end
  end

  class Ruby
    attr_reader :u

    def initialize(u, pid)
      @u = u
    end

    def receive_server
      @s_in = u.recv_io
      @s_ou = u.recv_io
      @s_output = u.recv_io
      u.send "o", 0
    end
    
    def to_a
      [@s_in, @s_ou, @s_output]
    end

    def run(cmd)
      puts "writing"
      write "RUN abc"
      puts @s_ou.readline
      write cmd
      write "END abc"
      puts @s_ou.readline
    end

    def write(line)
      @s_in.write line
      @s_in.write "\n"
      @s_in.flush
    end

    def output
      @s_output
    end

    def wait
      write "EXIT"
      puts @s_ou.readline
    end

    def fork
      write "FORK"
      puts @s_ou.readline

      r = Ruby.new(u, nil)
      r.receive_server
      r
    end
  end

  class Server
    def initialize(u)
      @in, parent_in = IO.pipe
      parent_out, @out = IO.pipe
      parent_cmd_out, @output = IO.pipe

      Client.send_pipes(u, parent_in, parent_out, parent_cmd_out)
      @u = u
    end

    def run
      @running = true
      @mode = :cmd
      @cmd_buffer = ""

      buffer = ""

      while @running
        begin
          buffer << @in.readpartial(1_024)
        rescue EOFError
          @running = false
        end

        if buffer.include?("\n")
          lines = buffer.split("\n")

          if buffer.end_with?("\n")
            buffer = ""
          else
            buffer = lines.last
            lines = lines[0..-2]
          end

          lines.each { |line| process_line(line) }
        end
      end

      output "EXIT"
    end

    def process_fork
      p,c = UNIXSocket.pair

      if pid = Kernel.fork
        r = Ruby.new(p, pid)
        r.receive_server
        r
        i,o,c = r.to_a
        Client.send_pipes(@u, i, o, c)
      else
        s = Forker::Server.new(c)
        s.run
        exit
      end
    end

    def process_command(line)
      case line.chomp
      when /^RUN (.+)$/
        output "OK RUN"
        @mode = :run
        @cmd_token = Regexp.last_match[1]
      when /^FORK/
        output "OK FORK"
        process_fork
      when /^QUIT|EXIT|LEAVE/
        output "OK #{line.chomp}"
        @running = false
      else
        puts "ERR unknown command #{line.chomp}"
        output "ERR unknown command"
      end
    end

    def process_cmd_buffer
      eval(@cmd_buffer)
      output "OK #{@cmd_token}"
    rescue => err
      output "ERR #{@cmd_token} #{err}"
    end

    def process_line(line)
      if @mode == :cmd
        @cmd_buffer = ""
        process_command(line)
      elsif line.start_with?("END #{@cmd_token}")
        process_cmd_buffer
        @cmd_buffer = ""
        @mode = :cmd
      else
        @cmd_buffer << line << "\n"
      end
    end

    def output(line)
      @out.write line
      @out.write "\n"
      @out.flush
    rescue Errno::EPIPE
    end
  end
end

client = Forker::Client.new

rspec_preamble = <<~CMD
  config = RSpec.configuration

  formatter = RSpec::Core::Formatters::DocumentationFormatter.new(config.output_stream)

  # create reporter with json formatter
  reporter =  RSpec::Core::Reporter.new(config)
  config.instance_variable_set(:@reporter, reporter)

  # internal hack
  # api may not be stable, make sure lock down Rspec version
  loader = config.send(:formatter_loader)
  notifications = loader.send(:notifications_for, RSpec::Core::Formatters::DocumentationFormatter)

  reporter.register_listener(formatter, *notifications)
CMD

client.run <<~CMD
  puts "loading env"
  ENV["RAILS_ENV"] = "test"
  require "./config/environment"
  require 'rspec'
  require 'rspec/core/formatters/documentation_formatter'
  require './spec/rails_helper'
  puts "env loaded"
CMD
client2 = client.fork
client2.run <<~CMD
  puts "reloading"
  begin
    reload!
  rescue => err
    puts err
  end
CMD

client2.run <<~CMD
begin
  puts "begin rspec run"
  #{rspec_preamble}
  RSpec::Core::Runner.run(['spec/models/user_spec.rb'])
rescue => err
  puts err
end
CMD
client2.wait
client.wait