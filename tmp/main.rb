require 'sinatra'
require 'logger'

logger = Logger.new(STDOUT)

get '/' do
  'get'
end

post '/' do
  body = request.body.read
  "body is #{body}"
end
