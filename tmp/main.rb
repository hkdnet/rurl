require 'sinatra'
require 'logger'

logger = Logger.new(STDOUT)

get '/' do
  'get'
end

post '/' do
  'post'
end
