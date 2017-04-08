require 'sinatra'
require 'logger'

logger = Logger.new(STDOUT)

get '/' do
  logger.info('Access to /')
  'Hello world!'
end
