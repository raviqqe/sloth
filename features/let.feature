Feature: Let expressions
  Background:
    Given a file named "ein.json" with:
    """
    {
      "target": {
        "type": "Command",
        "name": "foo"
      },
      "dependencies": {}
    }
    """

  Scenario: Use let-values expression
    Given a file named "Main.ein" with:
    """
    main : Number -> Number
    main x =
      let
        y : Number
        y = x
      in
        y
    """
    And I successfully run `ein build`
    When I run `sh -c ./foo`
    Then stdout from "sh -c ./foo" should contain exactly "42"
    And the exit status should be 0

  Scenario: Use untyped let-values expression
    Given a file named "Main.ein" with:
    """
    main : Number -> Number
    main x =
      let
        y = x
      in
        y
    """
    And I successfully run `ein build`
    When I run `sh -c ./foo`
    Then stdout from "sh -c ./foo" should contain exactly "42"
    And the exit status should be 0

  Scenario: Use nested let-values expression
    Given a file named "Main.ein" with:
    """
    main : Number -> Number
    main x =
      let
        y =
          let
            z = x
          in
            z
      in
        y
    """
    And I successfully run `ein build`
    When I run `sh -c ./foo`
    Then stdout from "sh -c ./foo" should contain exactly "42"
    And the exit status should be 0

  Scenario: Use let-functions expression
    Given a file named "Main.ein" with:
    """
    main : Number -> Number
    main x =
      let
        f : Number -> Number
        f y = y
      in
        f x
    """
    And I successfully run `ein build`
    When I run `sh -c ./foo`
    Then stdout from "sh -c ./foo" should contain exactly "42"
    And the exit status should be 0

  Scenario: Use untyped let-functions expression
    Given a file named "Main.ein" with:
    """
    main : Number -> Number
    main x =
      let
        f y = y
      in
        f x
    """
    And I successfully run `ein build`
    When I run `sh -c ./foo`
    Then stdout from "sh -c ./foo" should contain exactly "42"
    And the exit status should be 0

  Scenario: Define multiple functions in a let-functions expression
    Given a file named "Main.ein" with:
    """
    main : Number -> Number
    main x =
      let
        f y = y
        g z = f z
      in
        g x
    """
    And I successfully run `ein build`
    When I run `sh -c ./foo`
    Then stdout from "sh -c ./foo" should contain exactly "42"
    And the exit status should be 0