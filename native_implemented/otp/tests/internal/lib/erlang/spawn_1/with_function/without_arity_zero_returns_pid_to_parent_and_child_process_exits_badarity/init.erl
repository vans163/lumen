-module(init).
-export([start/0]).
-import(erlang, [display/1]).
-import(lumen, [log_exit/1]).

start() ->
  log_exit(false),
  ChildPid = spawn(fun (A) ->
    display(A)
  end),
  MonitorRef = monitor(process, ChildPid),
  receive
    {'DOWN', MonitorRef, process, _, Info} ->
      case Info of
        %% FIXME https://github.com/lumen/lumen/issues/548
        {Reason, FunArgs} -> display(Reason);
        Other -> display(Other)
      end
    after 20 ->
      display(timeout)
  end,
  ok.
