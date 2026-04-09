from __future__ import annotations


class ApplicationError(Exception):
    code = "internal_error"
    retriable = True
    status_code = 500

    def __init__(self, message: str):
        super().__init__(message)
        self.message = message


class RequestValidationError(ApplicationError):
    code = "invalid_request"
    retriable = False
    status_code = 400


class ProviderUnavailableError(ApplicationError):
    code = "provider_unavailable"
    retriable = True
    status_code = 503


class ProviderResponseError(ApplicationError):
    code = "provider_response_invalid"
    retriable = False
    status_code = 502


class AiUnavailableError(ApplicationError):
    code = "ai_unavailable"
    retriable = True
    status_code = 503


class AiRefusalError(ApplicationError):
    code = "ai_refused"
    retriable = False
    status_code = 422


class AiEmptyResponseError(ApplicationError):
    code = "ai_empty_response"
    retriable = True
    status_code = 502


class AiResponseFormatError(ApplicationError):
    code = "ai_response_invalid"
    retriable = False
    status_code = 502


class ExerciseResolutionError(ApplicationError):
    code = "exercise_resolution_failed"
    retriable = False
    status_code = 422
