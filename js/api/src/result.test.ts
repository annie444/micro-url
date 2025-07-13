import { test, expect } from "@jest/globals";
import { Result, ok, err, Err, Ok } from "../src/result";

test("ResultType Error", () => {
  class ExpectedError {
    readonly _tag = "ExpectedError";
  }

  function resultTest(): Result<string, ExpectedError> {
    const run = true;
    if (run) {
      return err(new ExpectedError());
    }
    return ok("Success");
  }

  let res2 = resultTest();
  expect(res2).toBeInstanceOf(Err);
  expect(res2._tag).toBe("err");
  res2 = res2 as Err<ExpectedError>;
  expect(res2.error).toBeInstanceOf(ExpectedError);
  expect(res2.error._tag).toBe("ExpectedError");
});

test("ResultType Success", () => {
  class ExpectedSuccess {
    readonly _tag = "ExpectedSuccess";
  }

  function resultTest(): Result<ExpectedSuccess, string> {
    const run = true;
    if (run) {
      return ok(new ExpectedSuccess());
    }
    return err("Error");
  }

  let res2 = resultTest();
  expect(res2).toBeInstanceOf(Ok);
  expect(res2._tag).toBe("ok");
  res2 = res2 as Ok<ExpectedSuccess>;
  expect(res2.value).toBeInstanceOf(ExpectedSuccess);
  expect(res2.value._tag).toBe("ExpectedSuccess");
});
