# frozen_string_literal: true

class Inverter
  def initialize
    @socket = TCPSocket.new('localhost', 8000)
  end

  def recv
    @socket.recv(4096).unpack('C*')
  end

  def write(bytes)
    @socket.write(bytes.pack('C*'))
  end
end
