-module(init).
-export([start/0]).

start() ->
  test:caught(fun () ->
    tl([])
  end).
