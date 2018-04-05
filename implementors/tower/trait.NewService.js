(function() {var implementors = {};
implementors["tower_h2"] = ["impl&lt;C, E, S&gt; <a class=\"trait\" href=\"https://docs.rs/tower/0.1/tower/trait.NewService.html\" title=\"trait tower::NewService\">NewService</a> for <a class=\"struct\" href=\"tower_h2/client/struct.Connect.html\" title=\"struct tower_h2::client::Connect\">Connect</a>&lt;C, E, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: Connect + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"https://docs.rs/futures/0.1/futures/future/trait.Executor.html\" title=\"trait futures::future::Executor\">Executor</a>&lt;<a class=\"struct\" href=\"tower_h2/client/struct.Background.html\" title=\"struct tower_h2::client::Background\">Background</a>&lt;C::Connected, S&gt;&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"tower_h2/trait.Body.html\" title=\"trait tower_h2::Body\">Body</a> + 'static,&nbsp;</span>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
