import textwrap


class Grader:
    def __init__(self):
        # max points
        self.http_request_parsing_max_points = 2
        self.http_response_parsing_max_points = 2

        self.http_method_get_max_points = 1.5
        self.http_method_post_max_points = 1.5
        self.http_method_put_max_points = 1.5
        self.http_method_delete_max_points = 1.5

        # passed test count
        self.http_request_parsing_passed_tests = 0
        self.http_response_parsing_passed_tests = 0

        self.http_method_get_passed_tests = 0
        self.http_method_post_passed_tests = 0
        self.http_method_put_passed_tests = 0
        self.http_method_delete_passed_tests = 0

        # test count
        self.http_request_parsing_total_tests = 0
        self.http_response_parsing_total_tests = 0

        self.http_method_get_total_tests = 0
        self.http_method_post_total_tests = 0
        self.http_method_put_total_tests = 0
        self.http_method_delete_total_tests = 0

    @property
    def request_parsing_mark(self):
        return self.http_request_parsing_max_points * (
            self.http_request_parsing_passed_tests
            / (self.http_request_parsing_total_tests or 1)
        )

    @property
    def response_parsing_mark(self):
        return self.http_response_parsing_max_points * (
            self.http_response_parsing_passed_tests
            / (self.http_response_parsing_total_tests or 1)
        )

    @property
    def method_get_mark(self):
        return self.http_method_get_max_points * (
            self.http_method_get_passed_tests / (self.http_method_get_total_tests or 1)
        )

    @property
    def method_post_mark(self):
        return self.http_method_post_max_points * (
            self.http_method_post_passed_tests
            / (self.http_method_post_total_tests or 1)
        )

    @property
    def method_put_mark(self):
        return self.http_method_put_max_points * (
            self.http_method_put_passed_tests / (self.http_method_put_total_tests or 1)
        )

    @property
    def method_delete_mark(self):
        return self.http_method_delete_max_points * (
            self.http_method_delete_passed_tests
            / (self.http_method_delete_total_tests or 1)
        )

    @property
    def total_tests(self):
        return (
            self.http_method_get_total_tests
            + self.http_method_put_total_tests
            + self.http_method_post_total_tests
            + self.http_method_delete_total_tests
            + self.http_response_parsing_total_tests
            + self.http_request_parsing_total_tests
        )

    @property
    def total_passed(self):
        return (
            self.http_method_get_passed_tests
            + self.http_method_put_passed_tests
            + self.http_method_post_passed_tests
            + self.http_method_delete_passed_tests
            + self.http_response_parsing_passed_tests
            + self.http_request_parsing_passed_tests
        )

    @property
    def grade(self):
        return (
            self.method_put_mark
            + self.method_get_mark
            + self.method_post_mark
            + self.method_delete_mark
            + self.request_parsing_mark
            + self.response_parsing_mark
        )

    def __str__(self):
        report = ""
        report += "=" * 80
        report += textwrap.dedent(
            f"""
            Grade results for http tests
            
            Total tests:            {self.total_tests}
            Total passed:           {self.total_passed}
            
            Request parsing
                Total  {self.http_request_parsing_total_tests}
                Passed {self.http_request_parsing_passed_tests}
                Mark   {self.request_parsing_mark:0.3f}
                
            Response parsing
                Total  {self.http_response_parsing_total_tests}
                Passed {self.http_response_parsing_passed_tests}
                Mark   {self.response_parsing_mark:0.3f}
                
            HTTP GET implementation
                Total  {self.http_method_get_total_tests}
                Passed {self.http_method_get_passed_tests}
                Mark   {self.method_get_mark:0.3f}
                
            HTTP POST implementation
                Total  {self.http_method_post_total_tests}
                Passed {self.http_method_post_passed_tests}
                Mark   {self.method_post_mark:0.3f}
                
            HTTP PUT implementation
                Total  {self.http_method_put_total_tests}
                Passed {self.http_method_put_passed_tests}
                Mark   {self.method_put_mark:0.3f}
                
            HTTP DELETE implementation
                Total  {self.http_method_delete_total_tests}
                Passed {self.http_method_delete_passed_tests}
                Mark   {self.method_delete_mark:0.3f}
                
            Final grade:
                {self.grade:0.3f}/10
        """
        )
        return report
