#! /usr/bin/env ruby
# frozen_string_literal: true

require 'socket'

server = TCPServer.new(8000)

clients = []

loop do
  rs, = IO.select(clients + [server])

  # try to avoid deadlocks by always accepting new connections before reading any data
  if rs.include?(server)
    clients << server.accept
    next
  end

  read_from = rs.first
  buf = read_from.recv(4096)

  if buf.empty?
    clients.delete_if { |c| read_from == c }
  else
    clients.reject { |c| read_from == c }.each { |client| client.write(buf) }
  end
end
