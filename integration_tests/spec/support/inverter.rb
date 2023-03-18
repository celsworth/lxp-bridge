# frozen_string_literal: true

class Inverter
  def initialize
    @messages = Queue.new
  end

  def run
    server = TCPServer.new(8000)

    @thread = Thread.new do
      wait_for_socket(server)
      go
    end
  end

  def recv
    messages.deq
  end

  def write(bytes)
    socket.write(bytes.pack('C*'))
  end

  def close
    socket&.close
    thread&.join
  end

  private

  attr_reader :messages, :socket, :thread

  def wait_for_socket(server)
    @socket = server.accept
  ensure
    server.close # No longer needed
  end

  def go
    loop do
      begin
        raw_data = socket.recv(4096)
      rescue IOError
        break # expected when the socket is closed
      end

      messages.enq(raw_data.unpack('C*'))
    end
  end
end
